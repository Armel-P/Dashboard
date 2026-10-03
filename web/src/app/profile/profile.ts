import { Component, inject, signal } from '@angular/core';
import { Router } from '@angular/router';
import { AuthService } from '../auth.service';

@Component({
  selector: 'app-profile',
  standalone: true,
  imports: [],
  templateUrl: './profile.html',
  styleUrl: './profile.css',
})
export class Profile {
  readonly auth = inject(AuthService);
  private readonly router = inject(Router);

  loading = signal(false);
  errorMessage = signal('');

  disconnect(): void {
    this.loading.set(true);
    this.errorMessage.set('');

    this.auth.disconnect()
      .subscribe({
        next: () => {
          this.router.navigate(['/']);
        },

        error: (error) => {
          this.loading.set(false);
          this.errorMessage.set(
            error?.error?.message ?? 'Failed to disconnect',
          );
        },
      });
  }

  deleteAccount(): void {
    if (!window.confirm(
      'Are you sure you want to delete your account? This action cannot be undone.'
    )) { return; };

    this.loading.set(true);
    this.errorMessage.set('');

    this.auth.deleteAccount()
      .subscribe({
        next: () => {
          this.router.navigate(['/']);
        },

        error: (error) => {
          this.loading.set(false);
          this.errorMessage.set(
            error?.error?.message ?? 'Failed to delete account.',
          );
        },
      });
  }
}
