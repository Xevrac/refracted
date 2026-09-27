<?php

namespace App\Models\Cdn;

use Illuminate\Database\Eloquent\Model;

class CdnRequestStat extends Model
{
    protected $table = 'cdn_request_stats';

    public $timestamps = false;

    protected $fillable = [
        'bucket_at',
        'title_id',
        'channel',
        'kind',
        'requests',
        'bytes_out',
        'status_2xx',
        'status_4xx',
        'status_5xx',
    ];

    protected function casts(): array
    {
        return [
            'bucket_at' => 'datetime',
            'requests' => 'integer',
            'bytes_out' => 'integer',
            'status_2xx' => 'integer',
            'status_4xx' => 'integer',
            'status_5xx' => 'integer',
        ];
    }
}
