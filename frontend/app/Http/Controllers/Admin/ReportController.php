<?php

namespace App\Http\Controllers\Admin;

use App\Http\Controllers\Controller;
use App\Models\ReportEvent;
use App\Models\ReportIssue;
use App\Support\GameReport;
use App\Support\SampleReport;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Http\Response;
use Illuminate\Support\Facades\Storage;
use Illuminate\View\View;

class ReportController extends Controller
{
    public function index(Request $request): View
    {
        SampleReport::ensure();

        $tab = $request->string('tab')->toString() === 'reporting' ? 'reporting' : 'issues';
        $filters = [
            'status' => $request->string('status')->toString() ?: 'unresolved',
            'type' => $request->string('type')->toString(),
            'game' => $request->string('game')->toString(),
            'q' => $request->string('q')->toString(),
        ];

        $scoped = ReportIssue::query()
            ->type($filters['type'])
            ->game($filters['game']);

        $issues = (clone $scoped)
            ->status($filters['status'] === 'all' ? null : $filters['status'])
            ->search($filters['q'])
            ->recent()
            ->paginate(25)
            ->withQueryString();

        $games = ReportIssue::query()
            ->whereNotNull('game')
            ->distinct()
            ->orderBy('game')
            ->pluck('game');

        return view('admin.reports.index', [
            'tab' => $tab,
            'issues' => $issues,
            'filters' => $filters,
            'types' => GameReport::TYPES,
            'games' => $games,
            'statusCounts' => (clone $scoped)
                ->selectRaw('status, count(*) as total')
                ->groupBy('status')
                ->pluck('total', 'status'),
            'report' => $this->reportingSummary(),
        ]);
    }

    /**
     * @return array{
     *     open: int,
     *     issues: int,
     *     events: int,
     *     games: int,
     *     byGame: \Illuminate\Support\Collection,
     *     byType: \Illuminate\Support\Collection
     * }
     */
    protected function reportingSummary(): array
    {
        $byGame = ReportIssue::query()
            ->selectRaw('game, count(*) as issues, sum(events_count) as events, sum(case when status = ? then 1 else 0 end) as open_issues, max(last_seen_at) as last_seen_at', ['unresolved'])
            ->groupBy('game')
            ->orderByDesc('issues')
            ->get();

        $byType = ReportIssue::query()
            ->selectRaw('type, count(*) as issues, sum(events_count) as events')
            ->groupBy('type')
            ->orderByDesc('issues')
            ->get();

        return [
            'open' => (int) ReportIssue::query()->where('status', 'unresolved')->count(),
            'issues' => (int) ReportIssue::query()->count(),
            'events' => (int) ReportIssue::query()->sum('events_count'),
            'games' => (int) $byGame->pluck('game')->filter()->unique()->count(),
            'byGame' => $byGame,
            'byType' => $byType,
        ];
    }

    public function show(Request $request, ReportIssue $issue): View
    {
        $events = $issue->events()
            ->orderByDesc('received_at')
            ->orderByDesc('id')
            ->paginate(10)
            ->withQueryString();

        // Default to the newest report; staff can pin a specific one via ?event=.
        $selected = $request->filled('event')
            ? $issue->events()->whereKey($request->integer('event'))->first()
            : $events->first();

        return view('admin.reports.show', [
            'issue' => $issue,
            'events' => $events,
            'event' => $selected,
        ]);
    }

    public function update(Request $request, ReportIssue $issue): RedirectResponse
    {
        $data = $request->validate([
            'status' => ['required', 'string', 'in:'.implode(',', ReportIssue::STATUSES)],
            'notes' => ['nullable', 'string', 'max:5000'],
        ]);

        $issue->forceFill([
            'status' => $data['status'],
            'notes' => $data['notes'] ?? null,
            'resolved_at' => $data['status'] === 'resolved' ? now() : null,
            'resolved_by' => $data['status'] === 'resolved' ? $request->user()->id : null,
        ])->save();

        return back()->with('status', 'Issue updated.');
    }

    public function destroy(ReportIssue $issue): RedirectResponse
    {
        // Attachments are removed by the event model's delete hook.
        $issue->events->each->delete();
        $issue->delete();

        return redirect()
            ->route('admin.reports.index')
            ->with('status', 'Issue deleted.');
    }

    /** Serve a report screenshot. */
    public function screenshot(ReportEvent $event): Response
    {
        abort_unless($event->hasScreenshot(), 404);

        $disk = Storage::disk(config('reports.disk'));

        abort_unless($disk->exists($event->screenshot_path), 404);

        return response($disk->get($event->screenshot_path), 200, [
            'Content-Type' => 'image/jpeg',
            'Cache-Control' => 'private, max-age=3600',
        ]);
    }

    public function raw(ReportEvent $event): Response
    {
        abort_unless(filled($event->raw_path), 404);

        $disk = Storage::disk(config('reports.disk'));

        abort_unless($disk->exists($event->raw_path), 404);

        return response($disk->get($event->raw_path), 200, [
            'Content-Type' => 'application/xml',
            'Content-Disposition' => 'attachment; filename="report-'.$event->id.'.xml"',
        ]);
    }
}
