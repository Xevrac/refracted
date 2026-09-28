<?php

namespace App\Models\Cdn;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

class CdnDownloadStat extends Model
{
    protected $table = 'cdn_download_stats';

    public $timestamps = false;

    protected $fillable = [
        'bucket_at',
        'package_id',
        'path',
        'downloads',
        'bytes_out',
    ];

    protected function casts(): array
    {
        return [
            'bucket_at' => 'datetime',
            'package_id' => 'integer',
            'downloads' => 'integer',
            'bytes_out' => 'integer',
        ];
    }

    public function package(): BelongsTo
    {
        return $this->belongsTo(CdnPackage::class, 'package_id');
    }
}
