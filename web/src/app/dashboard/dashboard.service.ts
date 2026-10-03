import { Injectable, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';

import { API_BASE } from '../config/api';
import { WidgetInstance } from '../widget/widget.types';

export interface DashboardMap {
  widgets: WidgetInstance[];
}

export interface UpdateMapRequest {
  map: DashboardMap;
}

@Injectable({
  providedIn: 'root',
})
export class DashboardService {
  private readonly http = inject(HttpClient);

  getMap(): Observable<DashboardMap> {
    return this.http.get<DashboardMap>('/api/user/get-map');
  }

  updateMap(map: DashboardMap): Observable<void> {
    return this.http.put<void>(
      '/api/user/update-map',
      { map }
    );
  }
}
