import {Component, effect, input, output, signal} from '@angular/core';

import {ParamDefinition, WidgetDefinition} from '../../widget/widget.types';

export interface WidgetConfig {
  params: Record<string, string | number>;
  refreshSecs: number;
}

type RefreshUnit =
  | 'seconds'
  | 'minutes'
  | 'hours'
  | 'days';

@Component({
  selector: 'app-widget-config-form',
  standalone: true,
  templateUrl: './widget_config_form.html',
  styleUrl: './widget_config_form.css',
})
export class WidgetConfigForm {
  widget = input.required<WidgetDefinition>();

  initialValues = input<Record<string, string | number>>({});
  initialRefreshSecs = input(60);

  changed = output<WidgetConfig>();

  protected readonly values =
    signal<Record<string, string | number>>({});

  protected readonly refreshValue = signal(60);

  protected readonly refreshUnit =
    signal<RefreshUnit>('seconds');

  protected readonly refreshUnits: {
    value: RefreshUnit;
    label: string;
  }[] = [
    { value: 'seconds', label: 'Seconds' },
    { value: 'minutes', label: 'Minutes' },
    { value: 'hours', label: 'Hours' },
    { value: 'days', label: 'Days' },
  ];

  constructor() {
    effect(() => {
      this.values.set({
        ...this.initialValues(),
      });

      this.setRefreshFromSeconds(
        this.initialRefreshSecs(),
      );
    });
  }

  protected setValue(
    p: ParamDefinition,
    raw: string,
  ): void {
    const next = { ...this.values() };

    if (raw === '') {
      delete next[p.name];
    } else if (p.type === 'string') {
      next[p.name] = raw;
    } else {
      const value = Number(raw);

      if (Number.isFinite(value)) {
        next[p.name] = value;
      } else {
        delete next[p.name];
      }
    }

    this.values.set(next);
    this.emitChanged();
  }

  protected setRefreshValue(raw: string): void {
    const value = Number(raw);

    if (!Number.isFinite(value) || value <= 0) {
      this.refreshValue.set(1);
    } else {
      this.refreshValue.set(value);
    }

    this.emitChanged();
  }

  protected setRefreshUnit(unit: string): void {
    if (!this.isRefreshUnit(unit))
      return;

    const currentSecs = this.refreshSecsValue();

    this.refreshUnit.set(unit);

    const converted = this.secondsToUnit(
      currentSecs,
      unit,
    );

    this.refreshValue.set(
      Number.isInteger(converted)
        ? converted
        : Number(converted.toFixed(2)),
    );

    this.emitChanged();
  }

  private emitChanged(): void {
    this.changed.emit({
      params: this.values(),
      refreshSecs: Math.max(
        5,
        this.refreshSecsValue(),
      ),
    });
  }

  private setRefreshFromSeconds(
    seconds: number,
  ): void {
    const safeSeconds = Math.max(5, seconds);

    if (safeSeconds % 86400 === 0) {
      this.refreshUnit.set('days');
      this.refreshValue.set(
        safeSeconds / 86400,
      );
    } else if (safeSeconds % 3600 === 0) {
      this.refreshUnit.set('hours');
      this.refreshValue.set(
        safeSeconds / 3600,
      );
    } else if (safeSeconds % 60 === 0) {
      this.refreshUnit.set('minutes');
      this.refreshValue.set(
        safeSeconds / 60,
      );
    } else {
      this.refreshUnit.set('seconds');
      this.refreshValue.set(
        safeSeconds,
      );
    }
  }

  private refreshSecsValue(): number {
    const value = Number(this.refreshValue());

    if (!Number.isFinite(value) || value <= 0)
      return 5;

    const multiplier: Record<
      RefreshUnit,
      number
    > = {
      seconds: 1,
      minutes: 60,
      hours: 60 * 60,
      days: 24 * 60 * 60,
    };

    return (
      value *
      multiplier[this.refreshUnit()]
    );
  }

  private secondsToUnit(
    seconds: number,
    unit: RefreshUnit,
  ): number {
    const divisor: Record<
      RefreshUnit,
      number
    > = {
      seconds: 1,
      minutes: 60,
      hours: 60 * 60,
      days: 24 * 60 * 60,
    };

    return seconds / divisor[unit];
  }

  private isRefreshUnit(
    value: string,
  ): value is RefreshUnit {
    return (
      value === 'seconds' ||
      value === 'minutes' ||
      value === 'hours' ||
      value === 'days'
    );
  }
}
