import { Routes } from '@angular/router';
import { Home } from './home/home';
import { Auth, AuthMode } from './auth/auth';
import { Dashboard } from './dashboard/dashboard';

export const routes: Routes = [
  { path: '', component: Home, pathMatch: 'full' },
  { path: 'login', component: Auth, data: { mode: AuthMode.Login } },
  { path: 'register', component: Auth, data: { mode: AuthMode.Register } },
  { path: 'dashboard', component: Dashboard },
  { path: '**', redirectTo: '' },
];