import {ApplicationConfig, inject, provideAppInitializer} from '@angular/core';
import {provideHttpClient, withInterceptors} from '@angular/common/http';
import {provideRouter} from '@angular/router';
import { routes } from './app.routes';
import { AuthService } from './auth.service';
import { authInterceptor } from './auth.interceptor';

export const appConfig: ApplicationConfig = {
  providers: [
    provideRouter(routes),

    provideHttpClient(
      withInterceptors([authInterceptor]),
    ),

    provideAppInitializer(() => {
      return inject(AuthService).initialize();
    }),
  ],
};
