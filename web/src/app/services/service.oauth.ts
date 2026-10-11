
import { Injectable, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';
import { API_BASE } from '../config/api';

export interface ConnectUrlResponse {
  url: string;
}

export interface ConnectedResponse {
  connected: boolean;
}

export interface ServiceCatalog {
  name: string;
  label: string;
  auth: unknown;
  widgets: Array<{
    id: string;
    name: string;
    description: string;
    params: unknown[];
  }>;
}

@Injectable({
  providedIn: 'root',
})
export class OAuthService {
  private readonly http = inject(HttpClient);

  connect(service: string): Observable<ConnectUrlResponse> {
    return this.http.post<ConnectUrlResponse>(
      `${API_BASE}/widgets/connect/${encodeURIComponent(service)}`,
      {},
    );
  }

  isConnected(service: string): Observable<ConnectedResponse> {
    return this.http.get<ConnectedResponse>(
      `${API_BASE}/widgets/connected/${encodeURIComponent(service)}`,
    );
  }

  disconnectService(service: string): Observable<void> {
    return this.http.delete<void>(
      `${API_BASE}/widgets/connect/${encodeURIComponent(service)}`,
    );
  }

  getCatalog(service: string): Observable<ServiceCatalog> {
    return this.http.get<ServiceCatalog>(
      `${API_BASE}/widgets/catalog/${encodeURIComponent(service)}`,
    );
  }
}
