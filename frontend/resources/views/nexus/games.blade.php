@extends('nexus.layout-account', ['title' => 'My Games', 'active' => 'games'])

@section('content')
    <div class="animate-fade">
        <p class="text-[11px] uppercase tracking-[0.18em] text-grit-mist">Nexus</p>
        <h1 class="mt-2 font-display text-3xl font-semibold tracking-[-0.03em] sm:text-4xl">My Games</h1>
        <p class="mt-3 max-w-xl text-sm leading-relaxed text-grit-mist">
            Titles linked to your Nexus persona through launcher sign-in.
            Your persona id stays the same across all of them.
        </p>
    </div>

    @if (! $nexusUser)
        <div class="mt-12 border border-amber-500/30 bg-amber-500/10 px-4 py-4 text-sm text-amber-100">
            No Nexus profile yet. Sign in with Discord from the profile page first.
        </div>
    @else
        <section class="mt-12 border border-grit-line bg-grit-surface/80 px-5 py-6 sm:px-6">
            @if ($games->isEmpty())
                <p class="text-sm text-grit-mist">
                    No games recorded yet. Launch a title through the Refracted launcher after approving a device code.
                </p>
            @else
                <div class="overflow-x-auto">
                    <table class="w-full min-w-[28rem] text-left text-sm">
                        <thead>
                            <tr class="border-b border-grit-line/80 text-[11px] uppercase tracking-[0.14em] text-grit-mist">
                                <th class="pb-3 pr-3 font-medium">Title</th>
                                <th class="pb-3 pr-3 font-medium">First seen</th>
                                <th class="pb-3 pr-3 font-medium">Last played</th>
                                <th class="pb-3 font-medium">Sessions</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-grit-line/60">
                            @foreach ($games as $game)
                                <tr>
                                    <td class="py-3 pr-3 font-display tracking-[-0.02em]">{{ $game['label'] }}</td>
                                    <td class="py-3 pr-3 text-grit-mist">{{ $game['first_seen'] }}</td>
                                    <td class="py-3 pr-3 text-grit-mist">{{ $game['last_seen'] }}</td>
                                    <td class="py-3 tabular-nums text-grit-mist">{{ number_format($game['sessions']) }}</td>
                                </tr>
                            @endforeach
                        </tbody>
                    </table>
                </div>
            @endif
        </section>
    @endif
@endsection
