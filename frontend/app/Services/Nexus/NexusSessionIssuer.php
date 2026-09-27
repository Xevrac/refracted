<?php

namespace App\Services\Nexus;

use App\Models\Nexus\NexusAuthSession;
use App\Models\Nexus\NexusPersona;
use App\Models\Nexus\NexusUser;
use App\Support\ClientGeo;
use Illuminate\Support\Facades\DB;

/** Issues Nexus game tickets. */
class NexusSessionIssuer
{
    public const SESSION_TTL_SECS = 259_200;

    public const GATEWAY_CLIENT_ID = 'GLACIER_LBGW_BK_OL_SERVER';

    /**
     * @param  array{client_ip?: ?string, country_code?: ?string, game_id?: ?string}  $meta
     */
    public function issue(NexusUser $user, NexusPersona $persona, array $meta = []): array
    {
        if ((int) $persona->user_id !== (int) $user->id) {
            throw new \InvalidArgumentException('Persona does not belong to that Nexus user.');
        }

        app(NexusAccessControl::class)->assertNotBanned(
            (string) ($user->discord_id ?? ''),
            (int) $user->id
        );

        $token = bin2hex(random_bytes(32));
        $jwtId = bin2hex(random_bytes(16));
        $tokenHash = $this->hashToken($token);
        $expiresAt = now()->utc()->addSeconds(self::SESSION_TTL_SECS);
        $now = now()->utc()->format('Y-m-d H:i:s');
        $jwt = $this->buildJwt($jwtId, (int) $user->id, (int) $persona->id, $persona->display_name);

        $clientIp = $this->nullableString($meta['client_ip'] ?? ClientGeo::ip());
        $country = $this->nullableCountry($meta['country_code'] ?? ClientGeo::countryCode());
        $gameId = $this->nullableGame($meta['game_id'] ?? null);

        $connection = config('nexus.connection', 'nexus');

        DB::connection($connection)->transaction(function () use (
            $user,
            $persona,
            $tokenHash,
            $jwtId,
            $expiresAt,
            $now,
            $clientIp,
            $country,
            $gameId
        ) {
            NexusAuthSession::query()->create([
                'user_id' => $user->id,
                'persona_id' => $persona->id,
                'token_hash' => $tokenHash,
                'jwt_id' => $jwtId,
                'expires_at' => $expiresAt->format('Y-m-d H:i:s'),
                'revoked_at' => null,
                'created_at' => $now,
                'last_seen_at' => $now,
                'client_ip' => $clientIp,
                'country_code' => $country,
                'game_id' => $gameId,
            ]);
        });

        return [
            'token' => $token,
            'jwt' => $jwt,
            'jwt_id' => $jwtId,
            'user_id' => (string) $user->id,
            'persona_id' => (string) $persona->id,
            'display_name' => $persona->display_name,
            'expires_at' => $expiresAt->toIso8601String(),
        ];
    }

    /**
     * Refresh telemetry when the launcher claims the ticket (better IP than browser approve).
     *
     * @param  array{client_ip?: ?string, country_code?: ?string, game_id?: ?string}  $meta
     */
    public function touchClientMeta(string $jwtId, array $meta = []): void
    {
        $jwtId = trim($jwtId);
        if ($jwtId === '') {
            return;
        }

        $update = [
            'last_seen_at' => now()->utc()->format('Y-m-d H:i:s'),
        ];

        $ip = $this->nullableString($meta['client_ip'] ?? null);
        if ($ip !== null) {
            $update['client_ip'] = $ip;
        }

        $country = $this->nullableCountry($meta['country_code'] ?? null);
        if ($country !== null) {
            $update['country_code'] = $country;
        }

        $game = $this->nullableGame($meta['game_id'] ?? null);
        if ($game !== null) {
            $update['game_id'] = $game;
        }

        NexusAuthSession::query()
            ->where('jwt_id', $jwtId)
            ->whereNull('revoked_at')
            ->update($update);
    }

    public function revokeByToken(string $token): void
    {
        if ($token === '') {
            return;
        }

        $hash = $this->hashToken($token);
        NexusAuthSession::query()
            ->where('token_hash', $hash)
            ->whereNull('revoked_at')
            ->update(['revoked_at' => now()->utc()->format('Y-m-d H:i:s')]);
    }

    public function hashToken(string $token): string
    {
        $pepper = (string) config('nexus.token_pepper', '');

        return $pepper === ''
            ? hash('sha256', $token)
            : hash('sha256', $pepper.':'.$token);
    }

    protected function buildJwt(string $jwtId, int $userId, int $personaId, string $displayName): string
    {
        $now = time();
        $header = ['alg' => 'HS256', 'typ' => 'JWT'];
        $psid = $personaId % 1_000_000_000;
        $payload = [
            'iss' => 'nexus.refracted.au',
            'jti' => $jwtId,
            'azp' => self::GATEWAY_CLIENT_ID,
            'iat' => $now,
            'exp' => $now + self::SESSION_TTL_SECS,
            'ver' => 1,
            'nexus' => [
                'cli' => self::GATEWAY_CLIENT_ID,
                'prd' => '5lkt',
                'pid' => (string) $personaId,
                'pty' => 'NUCLEUS',
                'uid' => (string) $userId,
                'psid' => $psid,
                'pltyp' => 'PC',
                'pnid' => 'EA',
                'dpid' => 'PC',
                'psif' => [[
                    'id' => $personaId,
                    'ns' => 'cem_ea_id',
                    'dis' => $displayName,
                    'nic' => $displayName,
                ]],
            ],
        ];

        $h = $this->b64(json_encode($header, JSON_UNESCAPED_SLASHES));
        $p = $this->b64(json_encode($payload, JSON_UNESCAPED_SLASHES));
        $key = (string) (config('nexus.jwt_key') ?: config('app.key'));
        $sig = $this->b64(hash_hmac('sha256', "{$h}.{$p}", $key, true));

        return "{$h}.{$p}.{$sig}";
    }

    protected function b64(string $raw): string
    {
        return rtrim(strtr(base64_encode($raw), '+/', '-_'), '=');
    }

    protected function nullableString(mixed $value): ?string
    {
        $value = trim((string) $value);

        return $value !== '' ? $value : null;
    }

    protected function nullableCountry(mixed $value): ?string
    {
        $value = strtoupper(trim((string) $value));
        if (preg_match('/^[A-Z]{2}$/', $value) !== 1 || in_array($value, ['XX', 'T1'], true)) {
            return null;
        }

        return $value;
    }

    protected function nullableGame(mixed $value): ?string
    {
        $value = strtolower(trim((string) $value));
        if ($value === '' || strlen($value) > 64 || preg_match('/^[a-z0-9_-]+$/', $value) !== 1) {
            return null;
        }

        return $value;
    }
}
