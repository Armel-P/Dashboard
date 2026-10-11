import { Component } from '@angular/core';
import { SharedDashboard } from '../shared';

@Component({
  selector: 'app-dashboard',
  standalone: true,
  imports: [SharedDashboard],
  templateUrl: './dashboard.html',
  styleUrl: './dashboard.css',
})
export class Dashboard {}
