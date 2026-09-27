<?php

namespace Tests\Unit\Cdn;

use App\Services\Cdn\CdnArtifactGuard;
use Illuminate\Validation\ValidationException;
use Tests\TestCase;

class CdnArtifactGuardTest extends TestCase
{
    private CdnArtifactGuard $guard;

    protected function setUp(): void
    {
        parent::setUp();
        $this->guard = new CdnArtifactGuard;
    }

    public function test_release_allows_plain_dll(): void
    {
        $info = $this->guard->inspect('prism.cnc.network.dll', 'release');
        $this->assertSame('ok', $info['severity']);
        $this->guard->assertMayUpload('prism.cnc.network.dll', 'release');
        $this->assertTrue(true);
    }

    public function test_release_blocks_pdb(): void
    {
        $info = $this->guard->inspect('prism.cnc.network.pdb', 'release');
        $this->assertSame('block', $info['severity']);
        $this->assertTrue($info['blocked_on_release']);

        $this->expectException(ValidationException::class);
        $this->guard->assertMayUpload('prism.cnc.network.pdb', 'release');
    }

    public function test_release_blocks_debug_named_dll(): void
    {
        $info = $this->guard->inspect('prism.cnc.debug.dll', 'release');
        $this->assertTrue($info['looks_debug']);
        $this->assertSame('block', $info['severity']);

        $this->expectException(ValidationException::class);
        $this->guard->assertMayUpload('prism.cnc.debug.dll', 'release');
    }

    public function test_release_blocks_exp_lib_map(): void
    {
        foreach (['foo.exp', 'foo.lib', 'foo.map'] as $name) {
            $this->assertSame('block', $this->guard->inspect($name, 'release')['severity'], $name);
        }
    }

    public function test_debug_allows_debug_dll_with_confirm(): void
    {
        $info = $this->guard->inspect('prism.cnc.debug.dll', 'debug');
        $this->assertSame('warn', $info['severity']);
        $this->assertTrue($info['requires_confirm']);

        $this->expectException(ValidationException::class);
        $this->guard->assertMayUpload('prism.cnc.debug.dll', 'debug', confirmed: false);
    }

    public function test_debug_accepts_debug_dll_when_confirmed(): void
    {
        $this->guard->assertMayUpload('prism.cnc.debug.dll', 'debug', confirmed: true);
        $this->assertTrue(true);
    }

    public function test_debug_still_blocks_exp(): void
    {
        $this->expectException(ValidationException::class);
        $this->guard->assertMayUpload('prism.exp', 'debug', confirmed: true);
    }

    public function test_publish_scan_lists_violations(): void
    {
        $violations = $this->guard->releaseViolations([
            'ok.dll',
            'bad.pdb',
            'prism.cnc.debug.dll',
        ]);
        $this->assertCount(2, $violations);
    }
}
