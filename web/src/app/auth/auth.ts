import { Component, inject, signal } from '@angular/core';
import { ActivatedRoute, Router, RouterLink } from '@angular/router';
import { FormBuilder, ReactiveFormsModule, ValidatorFn, Validators} from '@angular/forms';
import { AuthService } from '../auth.service';

export enum AuthMode {
  Login = 'login',
  Register = 'register',
}

const passwordsMatch: ValidatorFn = (group) => {
  const password = group.get('password')?.value;
  const confirmPassword = group.get('confirmPassword')?.value;

  if (!confirmPassword) {
    return null;
  }

    return password === confirmPassword ? null : { mismatch: true };
};

@Component({
  selector: 'app-auth',
  imports: [ReactiveFormsModule, RouterLink],
  templateUrl: './auth.html',
  styleUrl: './auth.css',
})
export class Auth {
  private readonly fb = inject(FormBuilder);
  private readonly route = inject(ActivatedRoute);
  private readonly router = inject(Router);
  private readonly auth = inject(AuthService);

  readonly mode: AuthMode = this.route.snapshot.data['mode'] ?? AuthMode.Login;

  readonly isRegister = this.mode === AuthMode.Register;

  loading = signal(false);
  errorMessage = signal('');

  form = this.fb.nonNullable.group(
    {
      name: ['', this.isRegister ? [Validators.required, Validators.pattern('^[a-zA-Z]+[ ]+[a-zA-Z]+$')] : []],
      email: ['', [Validators.required, Validators.email]],
      password: ['', [Validators.required, Validators.minLength(6)]],
      confirmPassword: [
        '',
        this.isRegister ? [Validators.required] : [],
      ],
    },
    {
      validators: [passwordsMatch],
    },
  );


  onSubmit(): void {
    if (this.form.invalid) {
      this.form.markAllAsTouched();
      return;
    }

    this.loading.set(true);
    this.errorMessage.set('');

    const { email, name, password } = this.form.getRawValue();

    if (this.isRegister) {
      this.auth
        .register({
          mail: email,
          name,
          password,
        })
        .subscribe({
          next: () => {
            this.router.navigate(['/dashboard']);
          },

          error: (error) => {
            this.loading.set(false);
            this.errorMessage.set(
              error?.error?.message ?? 'Registration failed.',
            );
          },
        });

      return;
    }

    this.auth
      .login({
        mail: email,
        password,
      })
      .subscribe({
        next: () => {
          this.router.navigate(['/dashboard']);
        },

        error: (error) => {
          this.loading.set(false);
          this.errorMessage.set(
            error?.error?.message ?? 'Invalid email or password.',
          );
        },
      });
  }
}
