<?php

namespace Database\Seeders;

use App\Models\Community;
use App\Models\Game;
use App\Models\User;
use Illuminate\Database\Seeder;
use Illuminate\Support\Facades\Hash;

class DatabaseSeeder extends Seeder
{
    public function run(): void
    {
        $this->seedLocalAdmin();
        $this->seedContent();
    }

    protected function seedLocalAdmin(): void
    {
        if (! app()->environment('local')) {
            return;
        }

        User::query()->updateOrCreate(
            ['email' => 'admin@refracted.local'],
            [
                'name' => 'Local Admin',
                'password' => Hash::make('password'),
                'email_verified_at' => now(),
                'is_admin' => true,
                'discord_username' => 'local-admin',
            ],
        );
    }

    protected function seedContent(): void
    {
        if (Game::query()->exists()) {
            return;
        }

        Game::query()->create([
            'name' => 'Command & Conquer',
            'status' => 'Coming soon',
            'blurb' => 'The classic strategy series, coming back for multiplayer through Aurora.',
            'image_path' => 'images/titles/command-conquer.png',
            'sort_order' => 1,
            'is_published' => true,
        ]);

        Game::query()->create([
            'name' => 'Battlefield Labs',
            'status' => 'Available',
            'blurb' => 'Play and experiment with Labs again, even without the official online setup.',
            'image_path' => 'images/titles/battlefield-labs.png',
            'sort_order' => 2,
            'is_published' => true,
        ]);

        Community::query()->create([
            'name' => 'Aurora',
            'tagline' => 'Powered by Refracted',
            'blurb' => 'Bringing Command & Conquer multiplayer back for players who still want to gather and compete.',
            'image_path' => 'images/communities/aurora-banner.png',
            'icon_path' => 'images/communities/aurora.png',
            'sort_order' => 1,
            'is_published' => true,
        ]);
    }
}
