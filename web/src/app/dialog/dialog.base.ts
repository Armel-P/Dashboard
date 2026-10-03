import {
  AfterViewInit,
  Directive,
  ElementRef,
  ViewChild,
} from '@angular/core';

@Directive()
export abstract class DialogBase implements AfterViewInit {
  @ViewChild('dlg', { static: true })
  protected dlg!: ElementRef<HTMLDialogElement>;

  protected dragging = false;
  protected dragOffsetX = 0;
  protected dragOffsetY = 0;

  ngAfterViewInit(): void {
    this.dlg.nativeElement.showModal();
  }

  protected startDrag(event: MouseEvent): void {
    if (event.button !== 0) return;

    const dialog = this.dlg.nativeElement;
    const rect = dialog.getBoundingClientRect();

    this.dragging = true;
    this.dragOffsetX = event.clientX - rect.left;
    this.dragOffsetY = event.clientY - rect.top;

    document.addEventListener('mousemove', this.onDrag);
    document.addEventListener('mouseup', this.stopDrag);
  }

  private onDrag = (event: MouseEvent): void => {
    if (!this.dragging) return;

    const dialog = this.dlg.nativeElement;

    dialog.style.left = `${event.clientX - this.dragOffsetX}px`;
    dialog.style.top = `${event.clientY - this.dragOffsetY}px`;
  };

  private stopDrag = (): void => {
    this.dragging = false;

    document.removeEventListener('mousemove', this.onDrag);
    document.removeEventListener('mouseup', this.stopDrag);
  };

  protected close(): void {
    this.dlg.nativeElement.close();
  }

  protected onBackdropClick(event: MouseEvent): void {
    if (event.target === this.dlg.nativeElement) {
      this.close();
    }
  }
}
