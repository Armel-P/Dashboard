import { Component, inject, signal, input, computed, output } from '@angular/core';
import { DecimalPipe } from '@angular/common';
import { EMPTY, switchMap, tap, timer, catchError } from 'rxjs';
import { toObservable, takeUntilDestroyed } from '@angular/core/rxjs-interop';

import { WidgetService } from '../widget.service';
import { CurrentWeather } from '../../services/open_meteo.service';

@Component({
  selector: 'app-open-meteo-current',
  standalone: true,
  imports: [DecimalPipe],
  templateUrl: './open_meteo_current.html',
  styleUrl: './open_meteo_current.css',
})
export class OpenMeteoCurrent {
  private readonly widgetService = inject(WidgetService);

  params = input.required<Record<string, string | number>>();
  refreshSecs = input.required<number>();

  weather = signal<CurrentWeather | null>(null);
  onModify = input<(() => void) | undefined>();
  loading = signal(true);
  error = signal('');

  constructor() {
    toObservable(computed(() => ({ p: this.params(), s: this.refreshSecs() }))).pipe(
      tap(() => this.loading.set(true)),
      switchMap(({ p, s }) =>
        timer(0, s * 1000).pipe(
          switchMap(() =>
            this.widgetService.getWidget('open_meteo', 'current', p).pipe(
              catchError(err => {
                this.error.set(err?.error?.message ?? 'Unable to load weather.');
                this.loading.set(false);
                return EMPTY;
              }),
            ),
          ),
        ),
      ),
      takeUntilDestroyed(),
    ).subscribe(data => {
      this.weather.set(data as CurrentWeather);
      this.error.set('');
      this.loading.set(false);
    });
  }
}