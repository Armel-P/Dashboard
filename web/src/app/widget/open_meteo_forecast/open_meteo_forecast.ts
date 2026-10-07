import { Component, inject, input, signal, computed } from '@angular/core';
import { DecimalPipe } from '@angular/common';
import { EMPTY, switchMap, tap, timer, catchError } from 'rxjs';
import { toObservable, takeUntilDestroyed } from '@angular/core/rxjs-interop';

import { WidgetService } from '../widget.service';
import { DailyForecast, ForecastDay } from '../../services/open_meteo.service';

@Component({
  selector: 'app-open-meteo-forecast',
  standalone: true,
  imports: [DecimalPipe],
  templateUrl: './open_meteo_forecast.html',
  styleUrl: './open_meteo_forecast.css',
})
export class OpenMeteoForecast {
  private readonly widgetService = inject(WidgetService);

  params = input.required<Record<string, string | number>>();
  refreshSecs = input.required<number>();
  onModify = input<(() => void) | undefined>();

  forecast = signal<DailyForecast | null>(null);
  loading = signal(true);
  error = signal('');

  constructor() {
    toObservable(
      computed(() => ({
        p: this.params(),
        s: this.refreshSecs(),
      })),
    ).pipe(
      tap(() => {
        this.loading.set(true);
        this.error.set('');
      }),
      switchMap(({ p, s }) =>
        timer(0, s * 1000).pipe(
          switchMap(() =>
            this.widgetService
              .getWidget('open_meteo', 'forecast', p)
              .pipe(
                catchError(err => {
                  this.error.set(
                    err?.error?.message ?? 'Unable to load forecast.',
                  );
                  this.loading.set(false);
                  return EMPTY;
                }),
              ),
          ),
        ),
      ),
      takeUntilDestroyed(),
    ).subscribe(data => {
      this.forecast.set(data as DailyForecast);
      this.error.set('');
      this.loading.set(false);
    });
  }

  formatDate(date: string): string {
    const parsed = new Date(`${date}T12:00:00`);

    return new Intl.DateTimeFormat(undefined, {
      weekday: 'short',
    }).format(parsed);
  }

  formatDateLong(date: string): string {
    const parsed = new Date(`${date}T12:00:00`);

    return new Intl.DateTimeFormat(undefined, {
      month: 'short',
      day: 'numeric',
    }).format(parsed);
  }

  weatherLabel(code: number): string {
    if (code === 0) return 'Clear';
    if ([1, 2, 3].includes(code)) return 'Cloudy';
    if ([45, 48].includes(code)) return 'Fog';
    if ([51, 53, 55, 56, 57].includes(code)) return 'Drizzle';
    if ([61, 63, 65, 66, 67].includes(code)) return 'Rain';
    if ([71, 73, 75, 77].includes(code)) return 'Snow';
    if ([80, 81, 82].includes(code)) return 'Showers';
    if ([85, 86].includes(code)) return 'Snow showers';
    if ([95, 96, 99].includes(code)) return 'Thunderstorm';

    return 'Unknown';
  }

  weatherIcon(code: number): string {
    if (code === 0) return '☀️';
    if ([1, 2].includes(code)) return '🌤️';
    if (code === 3) return '☁️';
    if ([45, 48].includes(code)) return '🌫️';
    if ([51, 53, 55, 56, 57].includes(code)) return '🌦️';
    if ([61, 63, 65, 66, 67].includes(code)) return '🌧️';
    if ([71, 73, 75, 77].includes(code)) return '❄️';
    if ([80, 81, 82].includes(code)) return '🌦️';
    if ([85, 86].includes(code)) return '🌨️';
    if ([95, 96, 99].includes(code)) return '⛈️';

    return '🌡️';
  }

  trackDay(_: number, day: ForecastDay): string {
    return day.date;
  }
}
