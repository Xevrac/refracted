<?php

namespace App\Support;

use Illuminate\Support\Str;

/**
 * A single report uploaded by the game client.
 *
 * The client serialises a flat XML document.
 */
class GameReport
{
    /** Report types the client can emit, used to reject junk early. */
    public const TYPES = [
        'crash',
        'desync',
        'session',
        'server',
        'game',
        'boot',
        'submit',
        'disconnect',
        'connectionfailure',
        'assert',
    ];

    /** @param array<string, string> $fields */
    private function __construct(
        public readonly array $fields,
        public readonly string $category,
        public readonly int $byteLength,
    ) {
    }

    /**
     * Parse an uploaded body. Returns null when this is not a report document.
     */
    public static function parse(string $body, string $category): ?self
    {
        $byteLength = strlen($body);

        // The client sends a fixed-size buffer, so the tail is usually NUL padding.
        $body = rtrim($body, "\0");
        $body = trim($body);

        if ($body === '') {
            return null;
        }

        $body = self::normaliseXml($body);

        if (Str::contains(Str::lower($body), '<devtrackbug')) {
            return self::parseDevTrack($body, $category, $byteLength);
        }

        if (! Str::contains($body, '<report')) {
            return null;
        }

        $xml = self::loadXml($body);

        if (! $xml instanceof \SimpleXMLElement || $xml->getName() !== 'report') {
            return null;
        }

        $fields = [];
        foreach ($xml->children() as $child) {
            $name = strtolower($child->getName());
            $value = trim((string) $child);

            if ($value !== '') {
                $fields[$name] = $value;
            }
        }

        return new self($fields, $category, $byteLength);
    }

    /**
     * Manual Bug Submit writes `DevTrackBug.xml`, not a BugSentry `<report>`.
     */
    private static function parseDevTrack(string $body, string $category, int $byteLength): ?self
    {
        $xml = self::loadXml($body);

        if (! $xml instanceof \SimpleXMLElement || strcasecmp($xml->getName(), 'DevTrackBug') !== 0) {
            return null;
        }

        $fields = [
            'type' => 'submit',
        ];

        foreach ($xml->children() as $child) {
            $name = strtolower($child->getName());

                if ($name === 'customfield') {
                $key = strtolower((string) preg_replace(
                    '/[^a-z0-9]+/i',
                    '',
                    trim((string) ($child->Key ?? $child->key ?? '')),
                ));
                $value = trim((string) ($child->Value ?? $child->value ?? ''));

                if ($key !== '' && $value !== '') {
                    $fields[$key] = $value;
                }

                continue;
            }

            $value = trim((string) $child);

            if ($value !== '') {
                $fields[$name] = $value;
            }
        }

        return new self(self::promoteDevTrack($fields), $category !== '' ? $category : 'submit', $byteLength);
    }

    /**
     * The Submit Bug dialog writes the assert into a Data field. Description is the extra note, often empty.
     *
     * @param  array<string, string>  $fields
     * @return array<string, string>
     */
    private static function promoteDevTrack(array $fields): array
    {
        foreach ($fields as $key => $value) {
            $normalised = strtolower((string) preg_replace('/[^a-z0-9]+/i', '', $key));

            if ($normalised !== '' && $normalised !== $key) {
                $fields[$normalised] = $value;
            }
        }

        $summary = $fields['title'] ?? '';
        $description = $fields['description'] ?? '';
        $data = $fields['data'] ?? '';

        if ($data === '') {
            foreach ($fields as $key => $value) {
                if (in_array($key, ['title', 'type', 'description', 'prismlog'], true)) {
                    continue;
                }

                if (stripos($value, 'Assert Info') !== false || stripos($value, 'Expression:') !== false) {
                    $data = $value;
                    break;
                }
            }
        }

        if ($data === '' && (stripos($description, 'Assert Info') !== false || stripos($description, 'Expression:') !== false)) {
            $data = $description;
        }

        $fileLine = self::extractFileLine($data);
        $function = self::labeledLine($data, 'Function');
        $isAssert = $fileLine !== null
            || $function !== null
            || stripos($data, 'Assert Info') !== false
            || stripos($data, 'Expression:') !== false;

        if ($isAssert) {
            $fields['type'] = 'assert';
        }

        if ($fileLine !== null) {
            $fields['categoryid'] = $function !== null ? $fileLine.' '.$function : $fileLine;
        } elseif ($summary !== '') {
            $fields['categoryid'] = $summary;
        }

        $context = $data;

        if ($description !== '' && $description !== $data) {
            $context = trim($context."\n\n".$description);
        }

        if ($context === '') {
            $context = $summary;
        }

        if ($isAssert) {
            $header = [];

            foreach (['severity' => 'Severity', 'assignto' => 'Assign To', 'changelist' => 'Changelist'] as $key => $label) {
                $value = $fields[$key] ?? '';

                if ($value !== '' && ! str_contains($context, $value)) {
                    $header[] = $label.': '.$value;
                }
            }

            if ($header !== []) {
                $context = implode("\n", $header)."\n\n".$context;
            }
        }

        if ($context !== '') {
            $fields['contextdata'] = $context;
        }

        $stack = self::extractCallstack($data);

        if ($stack !== null) {
            $fields['stack'] = $stack;
        }

        $build = $fields['build'] ?? '';

        if ($build !== '') {
            $fields['buildsignature'] = $build;
        }

        $session = $fields['juicesessionid'] ?? '';

        if ($session !== '') {
            $fields['sessionid'] = $session;
        }

        return $fields;
    }

    private static function extractFileLine(string $text): ?string
    {
        if (preg_match('/([A-Za-z0-9_.-]+\.(?:cpp|h|hpp|inl|c))\((\d+)\)/', $text, $match) !== 1) {
            return null;
        }

        return $match[1].':'.$match[2];
    }

    private static function labeledLine(string $text, string $label): ?string
    {
        if (preg_match('/'.preg_quote($label, '/').':\s*(.+)/i', $text, $match) !== 1) {
            return null;
        }

        $value = trim($match[1]);

        return $value !== '' ? $value : null;
    }

    private static function extractCallstack(string $text): ?string
    {
        if (preg_match('/Callstack:\s*(.+)\z/si', $text, $match) !== 1) {
            return null;
        }

        $stack = trim($match[1]);

        return $stack !== '' ? $stack : null;
    }

    private static function normaliseXml(string $body): string
    {
        $body = preg_replace('/encoding\s*=\s*["\']ISO-8859-1["\']/i', 'encoding="UTF-8"', $body) ?? $body;

        if (! mb_check_encoding($body, 'UTF-8')) {
            $converted = @mb_convert_encoding($body, 'UTF-8', 'ISO-8859-1');

            if (is_string($converted)) {
                return $converted;
            }
        }

        return $body;
    }

    private static function loadXml(string $body): ?\SimpleXMLElement
    {
        $previous = libxml_use_internal_errors(true);

        try {
            $xml = simplexml_load_string($body, 'SimpleXMLElement', LIBXML_NOCDATA | LIBXML_NONET);
        } finally {
            libxml_clear_errors();
            libxml_use_internal_errors($previous);
        }

        return $xml instanceof \SimpleXMLElement ? $xml : null;
    }

    public function field(string $name): ?string
    {
        return $this->fields[$name] ?? null;
    }

    public function game(): string
    {
        return self::gameSlug($this->field('sku') ?: $this->namedGame());
    }

    public static function gameSlug(?string $raw): string
    {
        $slug = strtolower(trim((string) $raw));
        $slug = preg_replace('/[^a-z0-9]+/', '-', $slug) ?? '';
        $slug = trim($slug, '-');

        return $slug !== '' ? substr($slug, 0, 64) : 'unknown';
    }

    public static function gameLabel(string $slug): string
    {
        return match ($slug) {
            'cnc', 'command-conquer', 'command-and-conquer' => 'Command & Conquer',
            'battlefield', 'bf' => 'Battlefield',
            'battlefield-labs', 'labs' => 'Battlefield Labs',
            'unknown' => 'Unknown',
            default => Str::headline(str_replace('-', ' ', $slug)),
        };
    }

    private function namedGame(): ?string
    {
        // DevTrack writes the sku into customfield_10041. customfield_10032 is the label "Game".
        foreach (['game', 'customfield10041', 'customfield_10041'] as $key) {
            $value = $this->field($key);

            if ($value === null) {
                continue;
            }

            $lower = strtolower(trim($value));

            if ($lower === '' || $lower === 'game' || in_array($lower, self::TYPES, true)) {
                continue;
            }

            return $value;
        }

        return null;
    }

    public function type(): string
    {
        $type = strtolower($this->field('type') ?? $this->category);

        return in_array($type, self::TYPES, true) ? $type : 'unknown';
    }

    public function categoryId(): ?string
    {
        $value = $this->field('categoryid');

        if ($value === null) {
            return null;
        }

        // Dedicated SEH dumps used to put the entire detail line in categoryid.
        // DB columns are varchar(255); keep the short head, full text stays in contextdata.
        return Str::limit($value, 240, '');
    }

    public function fingerprint(): string
    {
        $parts = [$this->game(), $this->type(), $this->categoryId() ?? $this->firstStackFrame()];

        if ($parts[1] === null) {
            $parts[] = Str::uuid()->toString();
        }

        return hash('sha256', implode('|', $parts));
    }

    /** Short label for the issue list. */
    public function title(): string
    {
        $categoryId = $this->categoryId();

        if ($categoryId !== null) {
            return Str::limit($categoryId, 180);
        }

        return match ($this->type()) {
            'desync' => 'Desync '.($this->field('desyncid') ?? 'report'),
            'server' => Str::limit($this->field('servererror') ?? 'Server error', 180),
            'assert' => Str::limit($this->field('categoryid') ?? 'Dedicated assert', 180),
            'submit' => Str::limit($this->field('title') ?? 'Bug submit', 180),
            default => Str::headline($this->type()).' report',
        };
    }

    /**
     * Where the fault happened
     */
    public function culprit(): ?string
    {
        $categoryId = $this->categoryId();

        if ($categoryId !== null && preg_match('/^(.+\.(?:cpp|h|hpp|inl|c):\d+)/', $categoryId, $match) === 1) {
            return $match[1];
        }

        return $this->firstStackFrame();
    }

    public function firstStackFrame(): ?string
    {
        $stack = $this->field('stack');

        if ($stack === null) {
            return null;
        }

        $frame = preg_split('/\s+/', trim($stack), 2)[0] ?? '';

        return $frame !== '' ? $frame : null;
    }

    /**
     * Client wall clock
     */
    public function createdAt(): ?\DateTimeImmutable
    {
        $raw = $this->field('createtime');

        if ($raw === null) {
            return null;
        }

        $parsed = \DateTimeImmutable::createFromFormat(
            'Y-m-d H:i:s',
            $raw,
            new \DateTimeZone('UTC'),
        );

        return $parsed ?: null;
    }

    /** Screenshots are base64 JPEG. Returns raw image bytes, or null. */
    public function screenshotBytes(): ?string
    {
        $encoded = $this->field('screenshot');

        if ($encoded === null) {
            return null;
        }

        $decoded = base64_decode(preg_replace('/\s+/', '', $encoded), true);

        if ($decoded === false || $decoded === '') {
            return null;
        }

        if (str_starts_with($decoded, "\xFF\xD8\xFF") || str_starts_with($decoded, "\x89PNG\r\n\x1a\n")) {
            return $decoded;
        }

        return null;
    }

    public function memDumpBytes(): ?string
    {
        $encoded = $this->field('memdump');

        if ($encoded === null) {
            return null;
        }

        $decoded = base64_decode(preg_replace('/\s+/', '', $encoded), true);

        return $decoded !== false && $decoded !== '' ? $decoded : null;
    }
}
