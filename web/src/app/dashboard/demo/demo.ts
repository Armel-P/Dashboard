import { Component, signal, inject } from '@angular/core';
import { SharedDashboard } from '../shared';

import { DashboardUiService } from '../dashboard_ui.service';

@Component({
  selector: 'app-demo',
  standalone: true,
  imports: [SharedDashboard],
  templateUrl: './demo.html',
  styleUrl: './demo.css',
})
export class Demo {
  protected readonly dashboardUiService = inject(DashboardUiService);
  readonly isEmpty = signal(true);
}
