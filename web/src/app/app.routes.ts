import { Routes } from '@angular/router';
import { guestGuard, authGuard } from './auth.guards';
import { Home } from './home/home';
import { Auth, AuthMode } from './auth/auth';
import { Dashboard } from './dashboard/dashboard';

export const routes: Routes = [
  {
    path: '',
    component: Home,
    pathMatch: 'full',
    canActivate: [guestGuard],
  },
  {
    path: 'login',
    component: Auth,
    data: { mode: AuthMode.Login },
    canActivate: [guestGuard],
  },
  {
    path: 'register',
    component: Auth,
    data: { mode: AuthMode.Register },
    canActivate: [guestGuard],
  },
  {
    path: 'dashboard',
    component: Dashboard,
    canActivate: [authGuard],
  },
  {
    path: '**',
    redirectTo: '',
  },
];
