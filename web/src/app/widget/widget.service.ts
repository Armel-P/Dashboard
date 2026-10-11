import { Injectable, inject } from "@angular/core";
import { HttpClient } from "@angular/common/http";
import { Observable, forkJoin, catchError, of, map } from "rxjs";

import { API_BASE } from "../config/api";
import { ServiceDefinition } from "./widget.types";

@Injectable({
  providedIn: 'root',
})
export class WidgetService {
  private readonly http = inject(HttpClient);

  getCatalog(service: string, freeMode: boolean): Observable<ServiceDefinition> {
    const endpoint = freeMode
      ? 'catalog-free'
      : 'catalog';

  return this.http.get<ServiceDefinition>(`${API_BASE}/widgets/${endpoint}/${service}`);
  }

  getCatalogs(services: string[], freeMode: boolean): Observable<ServiceDefinition[]> {
    if (services.length === 0) return of([]);

    return forkJoin(
      services.map(name =>
        this.getCatalog(name, freeMode).pipe(catchError(() => of(null))),
      ),
    ).pipe(
      map(list => list.filter((s): s is ServiceDefinition => s !== null)),
    );
  }

  getWidget(
    service: string,
    widgetId: string,
    params: Record<string, string | number>,
  ): Observable<unknown> {
    return this.http.post(
      `${API_BASE}/widgets/${service}/${widgetId}`,
      params,
    );
    // return this.http.request(
    //   'QUERY',
    //   `${API_BASE}/widgets/${service}/${widgetId}`,
    //   params,
    // );
    // hange to query method once implemented in the backend
  }
}
