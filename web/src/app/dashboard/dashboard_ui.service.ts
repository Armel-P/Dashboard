import { Injectable, signal } from '@angular/core';

@Injectable({ providedIn: 'root' })
export class DashboardUiService { addOpen = signal(false); }
