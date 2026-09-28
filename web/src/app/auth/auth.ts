import { Component, inject } from '@angular/core';
import { ActivatedRoute, RouterLink } from '@angular/router';
import { FormBuilder, ReactiveFormsModule, ValidatorFn, Validators } from '@angular/forms';

export enum AuthMode {
  Login = 'login',
  Register = 'register',
}

const passwordsMatch: ValidatorFn = (group) => {
  const { password, confirmPassword } = group.value;
  return confirmPassword && password !== confirmPassword ? { mismatch: true } : null;
};

@Component({
  selector: 'app-auth',
  imports: [ReactiveFormsModule, RouterLink],
  templateUrl: './auth.html',
  styleUrl: './auth.css',
})
export class Auth {
  private fb = inject(FormBuilder);
  private route = inject(ActivatedRoute);

  readonly mode: AuthMode = this.route.snapshot.data['mode'] ?? AuthMode.Login;
  readonly isRegister = this.mode === AuthMode.Register;

  form = this.fb.nonNullable.group(
    {
      email: ['', [Validators.required, Validators.email]],
      password: ['', [Validators.required, Validators.minLength(6)]],
      confirmPassword: ['', this.isRegister ? [Validators.required] : []],
    },
    { validators: [passwordsMatch] },
  );

  onSubmit() {
    if (this.form.invalid) {
      this.form.markAllAsTouched();
      return;
    }
    const { email, password } = this.form.getRawValue();
    console.log(this.isRegister ? 'register' : 'login', { email, password });
    // TODO: call the API
  }
}
