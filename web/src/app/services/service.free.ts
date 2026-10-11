
import { Injectable, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';
import { API_BASE } from '../config/api';

@Injectable({
  providedIn: 'root',
})
export class FreeWidgetService {
  private readonly http = inject(HttpClient);

  getWidget(
    service: string,
    widgetId: string,
    params: Record<string, string | number>,
  ): Observable<unknown> {
    return this.http.post(
      `${API_BASE}/widgets/free/${service}/${widgetId}`,
      params,
    );
  }
}
