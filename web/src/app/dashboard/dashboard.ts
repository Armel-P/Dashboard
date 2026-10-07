import { AfterViewInit, Component, ElementRef, ViewChild, ChangeDetectorRef,
  inject, signal } from '@angular/core';
import { NgComponentOutlet } from '@angular/common';
import { GridStack, GridStackOptions } from 'gridstack';

import { DashboardUiService } from './dashboard_ui.service';
import { DashboardService } from './dashboard.service';
import { WidgetInstance } from '../widget/widget.types';
import { WIDGET_REGISTRY } from '../widget/widget.registry';
import { AddWidgetDialog } from '../dialog/add_widget_dialog/add_widget_dialog';
import { ModifyWidgetDialog } from '../dialog/modify_widget_dialog/modify_widget_dialog';

export interface DashboardMap {
  widgets: WidgetInstance[];
}

@Component({
  selector: 'app-dashboard',
  standalone: true,
  imports: [NgComponentOutlet, AddWidgetDialog, ModifyWidgetDialog],
  templateUrl: './dashboard.html',
  styleUrl: './dashboard.css',
})
export class Dashboard
  implements AfterViewInit
{
  @ViewChild('grid', { static: true })
  private gridEl!: ElementRef<HTMLElement>;

  private cdr = inject(ChangeDetectorRef);
  private dashboardService = inject(DashboardService);
  protected ui = inject(DashboardUiService);

  widgets = signal<WidgetInstance[]>([]);
  editingWidget = signal<WidgetInstance | null>(null);

  private grid: GridStack | null = null;

  private saveTimer: ReturnType<typeof setTimeout> | null = null;

  private readonly gridOptions:
    GridStackOptions = {
      column: 12,
      cellHeight: 100,
      margin: 8,
      mode: 'float',
      draggable: {
        handle: '.widget-drag-handle',
      },
    };

  ngAfterViewInit(): void {
    this.grid = GridStack.init(
      this.gridOptions,
      this.gridEl.nativeElement,
    );

    this.setupGridEvents();
    this.loadFromServer();
  }

  private setupGridEvents(): void {
    this.grid?.on(
      'change',
      (_event, nodes) => {
        if (!nodes)
          return;

        this.widgets.update(current => {
          const updated = [...current];

          for (const node of nodes) {
            const uid = String(
              node.id ??
              node.el?.getAttribute(
                'gs-id',
              ) ??
              '',
            );
            if (!uid)
              continue;

            const index =
              updated.findIndex(
                w => w.uid === uid,
              );
            if (index === -1)
              continue;

            updated[index] = {
              ...updated[index],
              x: node.x,
              y: node.y,
              w: node.w ?? updated[index].w,
              h: node.h ?? updated[index].h,
            };
          }
          return updated;
        });
        this.scheduleSave();
      },
    );
  }

  componentFor(w: WidgetInstance) {
    return WIDGET_REGISTRY.find(
      entry =>
        entry.service === w.service &&
        entry.id === w.widgetId,
    )?.component ?? null;
  }

  add(w: WidgetInstance): void {
    this.widgets.update(
      list => [...list, w],
    );

    this.cdr.detectChanges();

    const el =
      this.gridEl.nativeElement
        .querySelector<HTMLElement>(
          `[gs-id="${w.uid}"]`,
        );
    if (el && this.grid) {
      this.grid.makeWidget(el, {
        minW: w.minW,
        minH: w.minH,
        w: w.w,
        h: w.h,
        x: w.x,
        y: w.y,
        id: w.uid,
      });
    }

    this.applyLockState(w);
    this.scheduleSave();
  }

  remove(uid: string): void {
    const el =
      this.gridEl.nativeElement
        .querySelector<HTMLElement>(
          `[gs-id="${uid}"]`,
        );
    if (el && this.grid)
      this.grid.removeWidget(el, false);

    this.widgets.update(
      list =>
        list.filter(
          w => w.uid !== uid,
        ),
    );

    this.scheduleSave();
  }

  toggleLock(uid: string): void {
    const widget = this.widgets()
      .find(w => w.uid === uid);
    if (!widget)
      return;

    const updated: WidgetInstance = {
      ...widget,
      locked: !widget.locked,
    };

    this.widgets.update(
      widgets =>
        widgets.map(w =>
          w.uid === uid
            ? updated
            : w,
        ),
    );

    this.applyLockState(updated);
    this.scheduleSave();
  }

  openModify(uid: string): void {
    const widget = this.widgets()
      .find(w => w.uid === uid);
    if (!widget)
      return;

    this.editingWidget.set({
      ...widget,
      params: {
        ...widget.params,
      },
    });
  }

  modify(updated: WidgetInstance): void {
    this.widgets.update(
      widgets =>
        widgets.map(widget =>
          widget.uid === updated.uid
            ? updated
            : widget,
        ),
    );

    this.applyLockState(updated);
    this.scheduleSave();
    this.editingWidget.set(null);
  }

  deleteFromDialog(uid: string): void {
    this.remove(uid);
    this.editingWidget.set(null);
  }

  closeModify(): void {
    this.editingWidget.set(null);
  }

  private applyLockState(
    widget: WidgetInstance,
  ): void {
    const el =
      this.gridEl.nativeElement
        .querySelector<HTMLElement>(
          `[gs-id="${widget.uid}"]`,
        );
    if (!el || !this.grid)
      return;

    this.grid.update(el, {
      locked: widget.locked,
      noMove: widget.locked,
      noResize: widget.locked,
    });
  }

  private scheduleSave(): void {
    if (this.saveTimer !== null) {
      clearTimeout(
        this.saveTimer,
      );
    }

    this.saveTimer =
      setTimeout(() => {
        this.saveTimer = null;
        this.saveToServer();
      }, 750);
  }

  private saveToServer(): void {
    const map: DashboardMap = {
      widgets: this.widgets(),
    };

    this.dashboardService
      .updateMap(map)
      .subscribe({
        error: error => {
          console.error(
            'Failed to save dashboard:',
            error,
          );
        },
      });
  }

  private loadFromServer(): void {
    this.dashboardService
      .getMap()
      .subscribe({
        next: map => {
          if (!map?.widgets)
            return;

          this.replaceWidgets(map.widgets);
        },
        error: error => {
          // Not any map saved yet
          if (error.status === 404)
            return;

          console.error(
            'Failed to load dashboard:',
            error,
          );
        },
      });
  }

  private replaceWidgets(
    widgets: WidgetInstance[],
  ): void {
    if (!this.grid)
      return;

    this.grid.removeAll(false);
    this.widgets.set(widgets);
    this.cdr.detectChanges();

    for (const widget of widgets) {
      const el =
        this.gridEl.nativeElement
          .querySelector<HTMLElement>(
            `[gs-id="${widget.uid}"]`,
          );
      if (!el)
        continue;

      this.grid.makeWidget(el, {
        id: widget.uid,
        x: widget.x,
        y: widget.y,
        w: widget.w,
        h: widget.h,
        minW: widget.minW,
        minH: widget.minH,
      });

      this.applyLockState(widget);
    }
  }
}
