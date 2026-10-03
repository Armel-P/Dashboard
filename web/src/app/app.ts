import { Component, inject } from '@angular/core';
import { RouterLink, RouterOutlet, Router, NavigationEnd } from '@angular/router';
import { filter, map } from 'rxjs';
import { ThemeService } from './theme.service';
import { toSignal } from '@angular/core/rxjs-interop';

import { AuthService } from './auth.service';
import { DashboardUiService } from './dashboard/dashboard_ui.service';

@Component({
  selector: 'app-root',
  imports: [RouterLink, RouterOutlet],
  templateUrl: './app.html',
  styleUrl: './app.css'
})
export class App {
  protected readonly authService = inject(AuthService);
  protected readonly themeService = inject(ThemeService);
  protected readonly dashboardUiService = inject(DashboardUiService);
  private readonly router = inject(Router);

  protected readonly isDashboard = toSignal(
    this.router.events.pipe(
      filter((e): e is NavigationEnd => e instanceof NavigationEnd),
      map(e => e.urlAfterRedirects.startsWith('/dashboard')),
    ),
    { initialValue: false },
  );

  constructor() {
    this.authService.initialize();
  }

  toggleTheme(): void {
    this.themeService.toggle();
  }
}
