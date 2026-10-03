import { Component, computed, effect, inject, input, output, signal } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';

import { WIDGET_REGISTRY, SERVICE_NAMES } from '../../widget/widget.registry';
import { WidgetService } from '../../widget/widget.service';
import { WidgetInstance } from '../../widget/widget.types';
import { DialogBase } from '../dialog.base';
import { WidgetConfig, WidgetConfigForm } from '../widget_config_form/widget_config_form';

@Component({
  selector: 'app-modify-widget-dialog',
  standalone: true,
  imports: [WidgetConfigForm],
  templateUrl: './modify_widget_dialog.html',
    styleUrls: [
    '../dialog.css',
    './modify_widget_dialog.css',
  ],
})
export class ModifyWidgetDialog extends DialogBase {
  private readonly widgetService = inject(WidgetService);

  instance = input.required<WidgetInstance>();
  saved = output<WidgetInstance>();
  deleted = output<string>();
  closed = output<void>();

  protected readonly catalog = toSignal(
    this.widgetService.getCatalogs(SERVICE_NAMES),
    { initialValue: null },
  );

  protected readonly config = signal<WidgetConfig | null>(null);
  protected readonly locked = signal(false);

  protected readonly widget = computed(() => {
    const instance = this.instance();

    return this.catalog()
      ?.find(
        service =>
          service.name === instance.service,
      )
      ?.widgets.find(
        widget =>
          widget.id === instance.widgetId,
      ) ?? null;
  });

  constructor() {
    super();

    effect(() => {
      const instance = this.instance();

      this.locked.set(instance.locked);

      this.config.set({
        params: {
          ...instance.params,
        },
        refreshSecs: instance.refreshSecs,
      });
    });
  }

  protected configChanged(config: WidgetConfig): void {
    this.config.set(config);
  }

  protected toggleLocked(): void {
    this.locked.update(
      value => !value,
    );
  }

  protected confirm(): void {
    const instance = this.instance();
    const config = this.config();

    if (!config)
      return;

    const updated: WidgetInstance = {
      ...instance,
      params: {
        ...config.params,
      },
      refreshSecs: config.refreshSecs,
      locked: this.locked(),
    };

    this.saved.emit(updated);
    this.close();
  }

  protected delete(): void {
    this.deleted.emit(
      this.instance().uid,
    );
    this.close();
  }
}
