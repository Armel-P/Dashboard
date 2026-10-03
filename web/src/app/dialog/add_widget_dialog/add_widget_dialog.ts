import {AfterViewInit, Component, ElementRef, ViewChild,
  computed, inject, output, signal} from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';

import {WIDGET_REGISTRY, SERVICE_NAMES} from '../../widget/widget.registry';
import { WidgetService } from '../../widget/widget.service';
import { WidgetInstance} from '../../widget/widget.types';
import { DialogBase } from '../dialog.base';
import { WidgetConfig, WidgetConfigForm } from '../widget_config_form/widget_config_form';

@Component({
  selector: 'app-add-widget-dialog',
  standalone: true,
  imports: [WidgetConfigForm],
  templateUrl: './add_widget_dialog.html',
  styleUrl: '../dialog.css',
})
export class AddWidgetDialog extends DialogBase {
  private readonly widgetService = inject(WidgetService);

  added = output<WidgetInstance>();
  closed = output<void>();

  protected readonly catalog = toSignal(
    this.widgetService.getCatalogs(SERVICE_NAMES),
    { initialValue: null },
  );

  protected readonly serviceName = signal<string | null>(null);
  protected readonly widgetId = signal<string | null>(null);
  protected readonly config = signal<WidgetConfig | null>(null);
  protected readonly service = computed(
    () =>
      this.catalog()?.find(
        s => s.name === this.serviceName(),
      ) ?? null,
  );

  protected readonly widgets = computed(() =>
    (this.service()?.widgets ?? []).filter(
      w => this.entry(
        this.serviceName()!,
        w.id,
      ),
    ),
  );

  protected readonly widget = computed(
    () =>
      this.widgets().find(
        w => w.id === this.widgetId(),
      ) ?? null,
  );

  protected selectService(name: string): void {
    this.serviceName.set(name || null);
    this.widgetId.set(null);
    this.config.set(null);
  }

  protected selectWidget(id: string): void {
    const widget =
      this.widgets().find(
        w => w.id === id,
      ) ?? null;

    this.widgetId.set(widget?.id ?? null);
    if (!widget) {
      this.config.set(null);
      return;
    }

    const values: Record<
      string,
      string | number
    > = {};

    for (const p of widget.params) {
      if (p.default === undefined || p.default === null)
        continue;

      if (p.type === 'string') {
        values[p.name] = String(p.default);
      } else {
        const value = Number(p.default);
        if (Number.isFinite(value))
          values[p.name] = value;
      }
    }

    const entry = this.entry(
      this.serviceName()!,
      widget.id,
    );

    this.config.set({
      params: values,
      refreshSecs:
        entry?.defaultRefreshSecs ?? 60,
    });
  }

  protected configChanged(
    config: WidgetConfig,
  ): void {
    this.config.set(config);
  }

  protected confirm(): void {
    const widget = this.widget();
    const service = this.serviceName();
    const config = this.config();

    const entry =
      widget && service
        ? this.entry(service, widget.id)
        : null;

    if (!widget || !service || !entry || !config)
      return;

    this.added.emit({
      uid: crypto.randomUUID(),

      service,
      widgetId: widget.id,

      params: config.params,
      refreshSecs: config.refreshSecs,

      locked: false,

      w: entry.w,
      h: entry.h,
    });

    this.close();
  }

  protected override close(): void {
    super.close();
  }

  private entry(
    service: string,
    id: string,
  ) {
    return WIDGET_REGISTRY.find(
      e =>
        e.service === service &&
        e.id === id,
    );
  }
}
