import { Component } from '@angular/core';
import { NgComponentOutlet } from '@angular/common';
import { GridstackComponent, GridstackItemComponent, nodesCB } from 'gridstack/dist/angular';
import { GridStackOptions } from 'gridstack';

@Component({
  selector: 'app-dashboard',
  imports: [NgComponentOutlet],
  templateUrl: './dashboard.html',
  styleUrl: './dashboard.css',
})
export class Dashboard {
  gridOptions: GridStackOptions = { column: 12, cellHeight: 100, margin: 8};
}