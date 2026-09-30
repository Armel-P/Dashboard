import { Component, inject } from '@angular/core';
import { AuthService } from '../auth.service';

@Component({ selector: 'app-profile', templateUrl: './profile.html' })
export class Profile {
  auth = inject(AuthService);

  logout() {  }
  deleteAccount() {
    if (confirm('Delete your account? This cannot be undone.')) { }
  }
}