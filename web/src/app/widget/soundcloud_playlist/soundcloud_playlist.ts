import { Component, inject, input, signal, computed } from '@angular/core';
import { EMPTY, switchMap, tap, timer, catchError } from 'rxjs';
import { toObservable, takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { DomSanitizer } from '@angular/platform-browser';

import { WidgetService } from '../widget.service';

interface SoundCloudTrack {
  id: number | string;
  title: string;
  artist: string | null;
  duration_ms: number | null;
  artwork_url: string | null;
  permalink_url: string | null;
  access: string | null;
  embed_url: string;
}

interface SoundCloudPlaylistData {
  playlist: {
    id: number | string;
    title: string;
    artwork_url: string | null;
    permalink_url: string | null;
    embed_url: string;
  };
  tracks: SoundCloudTrack[];
}

@Component({
  selector: 'app-soundcloud-playlist',
  standalone: true,
  templateUrl: './soundcloud_playlist.html',
  styleUrl: './soundcloud_playlist.css',
})
export class SoundCloudPlaylist {
  private readonly widgetService = inject(WidgetService);

  private readonly sanitizer = inject(DomSanitizer);

  embedUrl = computed(() => {
    const url = this.data()?.playlist.embed_url;
    if (!url || !url.startsWith('https://w.soundcloud.com/player/?')) return null;
    return this.sanitizer.bypassSecurityTrustResourceUrl(url);
  });

  params = input.required<Record<string, string | number>>();
  refreshSecs = input.required<number>();
  onModify = input<(() => void) | undefined>();

  data = signal<SoundCloudPlaylistData | null>(null);
  loading = signal(true);
  error = signal('');

  constructor() {
    toObservable(
      computed(() => ({
        params: this.params(),
        refreshSecs: this.refreshSecs(),
      })),
    )
      .pipe(
        tap(() => {
          this.loading.set(true);
          this.error.set('');
        }),
        switchMap(({ params, refreshSecs }) =>
          timer(0, refreshSecs * 1000).pipe(
            switchMap(() =>
              this.widgetService
                .getWidget('soundcloud', 'playlist', params)
                .pipe(
                  catchError(err => {
                    this.error.set(
                      err?.error?.message ??
                        'Unable to load the SoundCloud playlist.',
                    );
                    this.loading.set(false);
                    return EMPTY;
                  }),
                ),
            ),
          ),
        ),
        takeUntilDestroyed(),
      )
      .subscribe(result => {
        this.data.set(result as SoundCloudPlaylistData);
        this.error.set('');
        this.loading.set(false);
      });
  }

  formatDuration(durationMs: number | null): string {
    if (durationMs == null || !Number.isFinite(durationMs)) {
      return '--:--';
    }

    const totalSeconds = Math.floor(durationMs / 1000);
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;

    return `${minutes}:${String(seconds).padStart(2, '0')}`;
  }

  trackById(_: number, track: SoundCloudTrack): string | number {
    return track.id;
  }
}
