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
                $key = strtolower(trim((string) ($child->Key ?? $child->key ?? '')));
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

        if (isset($fields['title'])) {
            $fields['categoryid'] = $fields['title'];
        }

        if (isset($fields['description'])) {
            $fields['contextdata'] = $fields['description'];
        }

        return new self($fields, $category !== '' ? $category : 'submit', $byteLength);
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

    public function type(): string
    {
        $type = strtolower($this->field('type') ?? $this->category);

        return in_array($type, self::TYPES, true) ? $type : 'unknown';
    }

    public function categoryId(): ?string
    {
        return $this->field('categoryid');
    }

    public function fingerprint(): string
    {
        $parts = [$this->type(), $this->categoryId() ?? $this->firstStackFrame()];

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

        if ($categoryId !== null && preg_match('/^(.+):(\d+)$/', $categoryId) === 1) {
            return $categoryId;
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

        // JPEG SOI. Anything else is not something we should hand to a browser.
        return str_starts_with($decoded, "\xFF\xD8\xFF") ? $decoded : null;
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
