<?php

namespace App\Services\Nexus;

use App\Support\ClientGeo;
use App\Support\NexusHosts;
use Illuminate\Contracts\Cache\LockTimeoutException;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Str;

/** Device-code flow for the launcher. */
class NexusDeviceCodeService
{
    public const INTERVAL = 5;

    public const EXPIRES_IN = 900;

    public function __construct(
        protected NexusSessionIssuer $sessions,
        protected NexusProvisioner $provisioner,
    ) {}

    public function start(?string $gameId = null): array
    {
        $deviceCode = bin2hex(random_bytes(24));
        $userCode = $this->newUserCode();
        $payload = [
            'device_code' => $deviceCode,
            'user_code' => $userCode,
            'status' => 'pending',
            'game_id' => $this->normalizeGameId($gameId),
            'created_at' => now()->utc()->timestamp,
            'expires_at' => now()->utc()->addSeconds(self::EXPIRES_IN)->timestamp,
            'credentials' => null,
        ];

        Cache::put($this->deviceKey($deviceCode), $payload, self::EXPIRES_IN);
        Cache::put($this->userKey($userCode), $deviceCode, self::EXPIRES_IN);

        return [
            'userCode' => $userCode,
            'deviceCode' => $deviceCode,
            'interval' => self::INTERVAL,
            'expiresIn' => self::EXPIRES_IN,
            'verificationUri' => $this->verificationUri($userCode),
        ];
    }

    /** Browser-safe view of a pending device request (never includes credentials). */
    public function publicByUserCode(string $userCode): ?array
    {
        $row = $this->findRawByUserCode($userCode);
        if ($row === null) {
            return null;
        }

        return [
            'user_code' => $row['user_code'] ?? null,
            'status' => $row['status'] ?? 'pending',
            'expires_at' => $row['expires_at'] ?? null,
        ];
    }

    public function approve(string $userCode, $websiteUser): array
    {
        $lock = Cache::lock('nexus:approve:'.Str::upper(trim($userCode)), 10);

        try {
            $lock->block(5);

            $row = $this->findRawByUserCode($userCode);
            if ($row === null) {
                throw new \RuntimeException('Unknown or expired device code.');
            }

            if (($row['expires_at'] ?? 0) < now()->utc()->timestamp) {
                throw new \RuntimeException('Device code expired.');
            }

            $status = $row['status'] ?? '';
            if ($status === 'approved' || $status === 'consumed') {
                return [
                    'user_code' => $row['user_code'] ?? null,
                    'status' => $status,
                ];
            }

            $nexusUser = $this->provisioner->ensureForWebsiteUser($websiteUser);
            $persona = $nexusUser->defaultPersona();
            if ($persona === null) {
                throw new \RuntimeException('Nexus persona missing.');
            }

            $issued = $this->sessions->issue($nexusUser, $persona, [
                'client_ip' => ClientGeo::ip(),
                'country_code' => ClientGeo::countryCode(),
                'game_id' => $row['game_id'] ?? null,
            ]);
            $row['status'] = 'approved';
            $row['credentials'] = $issued;
            $row['approved_at'] = now()->utc()->timestamp;
            $row['approved_web_user_id'] = $websiteUser->id;

            $ttl = max(60, ($row['expires_at'] ?? 0) - now()->utc()->timestamp);
            Cache::put($this->deviceKey($row['device_code']), $row, $ttl);
            Cache::put($this->userKey($row['user_code']), $row['device_code'], $ttl);

            return [
                'user_code' => $row['user_code'],
                'status' => 'approved',
            ];
        } catch (LockTimeoutException) {
            throw new \RuntimeException('Could not approve right now. Try again.');
        } finally {
            optional($lock)->release();
        }
    }

    public function poll(string $deviceCode): array
    {
        $lock = Cache::lock('nexus:poll:'.$deviceCode, 10);

        try {
            $lock->block(5);

            $row = Cache::get($this->deviceKey($deviceCode));
            if (! is_array($row)) {
                return ['status' => 'expired'];
            }

            if (($row['expires_at'] ?? 0) < now()->utc()->timestamp) {
                Cache::forget($this->deviceKey($deviceCode));
                Cache::forget($this->userKey($row['user_code'] ?? ''));

                return ['status' => 'expired'];
            }

            $status = $row['status'] ?? 'pending';
            if ($status === 'consumed') {
                return ['status' => 'expired'];
            }
            if ($status !== 'approved') {
                return ['status' => 'pending'];
            }

            $creds = $row['credentials'] ?? null;
            $row['status'] = 'consumed';
            $row['credentials'] = null;
            Cache::put($this->deviceKey($deviceCode), $row, 60);
            Cache::forget($this->userKey($row['user_code'] ?? ''));

            if (! is_array($creds) || empty($creds['token']) || empty($creds['jwt'])) {
                return ['status' => 'expired'];
            }

            if (! empty($creds['jwt_id'])) {
                $this->sessions->touchClientMeta((string) $creds['jwt_id'], [
                    'client_ip' => ClientGeo::ip(),
                    'country_code' => ClientGeo::countryCode(),
                    'game_id' => $row['game_id'] ?? null,
                ]);
            }

            return [
                'status' => 'approved',
                'token' => $creds['token'],
                'jwt' => $creds['jwt'],
                'displayName' => $creds['display_name'],
                'userId' => $creds['user_id'],
                'personaId' => $creds['persona_id'],
            ];
        } catch (LockTimeoutException) {
            return ['status' => 'pending'];
        } finally {
            optional($lock)->release();
        }
    }

    protected function findRawByUserCode(string $userCode): ?array
    {
        $deviceCode = Cache::get($this->userKey(Str::upper(trim($userCode))));
        if (! is_string($deviceCode) || $deviceCode === '') {
            return null;
        }

        $row = Cache::get($this->deviceKey($deviceCode));

        return is_array($row) ? $row : null;
    }

    protected function newUserCode(): string
    {
        // 10 chars from 32 ≈ 50 bits.
        $alphabet = 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789';
        do {
            $raw = '';
            for ($i = 0; $i < 10; $i++) {
                $raw .= $alphabet[random_int(0, strlen($alphabet) - 1)];
            }
            $code = substr($raw, 0, 5).'-'.substr($raw, 5, 5);
        } while (Cache::has($this->userKey($code)));

        return $code;
    }

    protected function verificationUri(string $userCode): string
    {
        return NexusHosts::deviceUrl($userCode);
    }

    protected function normalizeGameId(?string $gameId): ?string
    {
        $gameId = strtolower(trim((string) $gameId));
        if ($gameId === '' || strlen($gameId) > 64 || preg_match('/^[a-z0-9_-]+$/', $gameId) !== 1) {
            return null;
        }

        return $gameId;
    }

    protected function deviceKey(string $deviceCode): string
    {
        return 'nexus:device:'.$deviceCode;
    }

    protected function userKey(string $userCode): string
    {
        return 'nexus:user:'.Str::upper($userCode);
    }
}
