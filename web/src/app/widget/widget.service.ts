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

  getCatalog(service: string): Observable<ServiceDefinition> {
    return this.http.get<ServiceDefinition>(`${API_BASE}/widgets/catalog/${service}`);
  }

  getCatalogs(services: string[]): Observable<ServiceDefinition[]> {
    if (services.length === 0) return of([]);

    return forkJoin(
      services.map(name =>
        this.getCatalog(name).pipe(catchError(() => of(null))),
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
