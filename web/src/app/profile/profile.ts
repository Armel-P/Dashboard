import { Component, inject, signal, OnInit } from '@angular/core';
import { Router } from '@angular/router';
import { forkJoin } from 'rxjs';
import { catchError, finalize, of } from 'rxjs';

import { AuthService } from '../auth.service';
import { OAuthService } from '../services/service.oauth';
import { OAUTH_SERVICE_NAMES } from '../widget/widget.registry';

type ConnectionState = 'loading' | 'connected' | 'disconnected' | 'error';

@Component({
  selector: 'app-profile',
  standalone: true,
  imports: [],
  templateUrl: './profile.html',
  styleUrl: './profile.css',
})
export class Profile implements OnInit {
  readonly auth = inject(AuthService);

  private readonly router = inject(Router);
  private readonly oauth = inject(OAuthService);

  readonly loading = signal(false);
  readonly errorMessage = signal('');

  readonly oauthServices = OAUTH_SERVICE_NAMES;
  readonly connectionStates = signal<Record<string, ConnectionState>>({});
  readonly serviceErrors = signal<Record<string, string>>({});
  readonly serviceActions = signal<Record<string, boolean>>({});

  ngOnInit(): void {
    this.loadConnectionStates();
  }

  loadConnectionStates(): void {
    const services = this.oauthServices;

    if (services.length === 0) {
      this.connectionStates.set({});
      return;
    }

    this.connectionStates.set(
      Object.fromEntries(services.map(service => [service, 'loading'])),
    );
    this.serviceErrors.set({});

    forkJoin(
      services.map(service =>
        this.oauth.isConnected(service).pipe(
          catchError(error =>
            of({
              connected: null,
              error:
                error?.error?.message ??
                `Failed to check ${service} connection.`,
            }),
          ),
        ),
      ),
    ).subscribe(results => {
      const states: Record<string, ConnectionState> = {};
      const errors: Record<string, string> = {};

      results.forEach((result, index) => {
        const service = services[index];

        if ('error' in result) {
          states[service] = 'error';
          errors[service] = result.error;
        } else {
          states[service] = result.connected
            ? 'connected'
            : 'disconnected';
        }
      });

      this.connectionStates.set(states);
      this.serviceErrors.set(errors);
    });
  }

  connectService(service: string): void {
    this.setServiceAction(service, true);
    this.setServiceError(service, '');

    this.oauth.connect(service)
      .pipe(finalize(() => this.setServiceAction(service, false)))
      .subscribe({
        next: response => {
          window.location.assign(response.url);
        },
        error: error => {
          this.setServiceError(
            service,
            error?.error?.message ?? `Failed to connect ${service}.`,
          );
        },
      });
  }

  disconnectService(service: string): void {
    this.setServiceAction(service, true);
    this.setServiceError(service, '');

    this.oauth.disconnectService(service)
      .pipe(finalize(() => this.setServiceAction(service, false)))
      .subscribe({
        next: () => {
          this.connectionStates.update(states => ({
            ...states,
            [service]: 'disconnected',
          }));
        },
        error: error => {
          this.setServiceError(
            service,
            error?.error?.message ?? `Failed to disconnect ${service}.`,
          );
        },
      });
  }

  retryConnectionCheck(service: string): void {
    this.connectionStates.update(states => ({
      ...states,
      [service]: 'loading',
    }));

    this.setServiceError(service, '');

    this.oauth.isConnected(service).subscribe({
      next: response => {
        this.connectionStates.update(states => ({
          ...states,
          [service]: response.connected ? 'connected' : 'disconnected',
        }));
      },
      error: error => {
        this.connectionStates.update(states => ({
          ...states,
          [service]: 'error',
        }));
        this.setServiceError(
          service,
          error?.error?.message ?? `Failed to check ${service} connection.`,
        );
      },
    });
  }

  private setServiceAction(service: string, active: boolean): void {
    this.serviceActions.update(actions => ({
      ...actions,
      [service]: active,
    }));
  }

  private setServiceError(service: string, message: string): void {
    this.serviceErrors.update(errors => {
      const updated = { ...errors };

      if (message) {
        updated[service] = message;
      } else {
        delete updated[service];
      }

      return updated;
    });
  }

  disconnect(): void {
    this.loading.set(true);
    this.errorMessage.set('');

    this.auth.disconnect().subscribe({
      next: () => {
        this.router.navigate(['/']);
      },
      error: error => {
        this.loading.set(false);
        this.errorMessage.set(
          error?.error?.message ?? 'Failed to disconnect',
        );
      },
    });
  }

  deleteAccount(): void {
    if (!window.confirm(
      'Are you sure you want to delete your account? This action cannot be undone.',
    )) {
      return;
    }

    this.loading.set(true);
    this.errorMessage.set('');

    this.auth.deleteAccount().subscribe({
      next: () => {
        this.router.navigate(['/']);
      },
      error: error => {
        this.loading.set(false);
        this.errorMessage.set(
          error?.error?.message ?? 'Failed to delete account.',
        );
      },
    });
  }
}
