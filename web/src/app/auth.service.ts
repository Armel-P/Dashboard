import { Injectable, inject, signal } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { API_BASE } from './config/api';
import { Observable, of, catchError, tap } from 'rxjs';

export interface UserInfo {
  id: string;
  mail: string;
  name: string;
}

export interface AuthResponse {
  access_token: string;
  user: UserInfo;
}

export interface LoginRequest {
  mail: string;
  password: string;
}

export interface RegisterRequest {
  mail: string;
  name: string;
  password: string;
}

@Injectable({
  providedIn: 'root',
})
export class AuthService {
  private readonly http = inject(HttpClient);

  private readonly _token = signal<string | null>(null);
  private readonly _user = signal<UserInfo | null>(null);
  private readonly _initialized = signal(false);

  readonly token = this._token.asReadonly();
  readonly user = this._user.asReadonly();
  readonly initialized = this._initialized.asReadonly();

  get isAuthenticated(): boolean {
    return this._token() !== null;
  }

  login(request: LoginRequest): Observable<AuthResponse> {
    return this.http
      .post<AuthResponse>(
        `${API_BASE}/auth/login`,
        request,
        { withCredentials: true },
      )
      .pipe(
        tap(({ access_token, user }) => {
          this._token.set(access_token);
          this._user.set(user);
        }),
      );
  }

  register(request: RegisterRequest): Observable<AuthResponse> {
    return this.http
      .post<AuthResponse>(
        `${API_BASE}/auth/register`,
        request,
        { withCredentials: true },
      )
      .pipe(
        tap(({ access_token, user }) => {
          this._token.set(access_token);
          this._user.set(user);
        }),
      );
  }

  initialize(): Observable<AuthResponse | null> {
    return this.http
      .post<AuthResponse>(
        `${API_BASE}/auth/get_jwt`,
        {},
        { withCredentials: true },
      )
      .pipe(
        tap(({ access_token, user }) => {
          this._token.set(access_token);
          this._user.set(user);
        }),
        catchError(() => {
          this._token.set(null);
          this._user.set(null);
          return of(null);
        }),
        tap(() => {
          this._initialized.set(true);
        }),
      );
  }


  logout(): void {
    this._token.set(null);
    this._user.set(null);
  }
}
