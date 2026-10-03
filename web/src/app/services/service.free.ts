import { Injectable, inject } from "@angular/core";
import { HttpClient, HttpParams } from "@angular/common/http";
import { Observable } from "rxjs";
import { API_BASE } from "../config/api";

@Injectable({
  providedIn: 'root',
})
export class WidgetService {
  private readonly http = inject(HttpClient);

  getWidget(
    service: string,
    widgetId: string,
    params?: Record<string, string | number>,
  ): Observable<unknown> {
    let httpParams = new HttpParams();

    for (const [key, value] of Object.entries(params ?? {})) {
      httpParams = httpParams.set(key, String(value));
    }

    return this.http.get(
      `${API_BASE}/widgets/${service}/${widgetId}`,
      { params: httpParams },
    );
  }
}
