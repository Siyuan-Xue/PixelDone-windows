export type AuthMode = 'sign-in' | 'sign-up';

export type AuthValidationError = 'required' | 'invalid-email' | 'password-short';

const emailPattern = /^[^\s@]+@[^\s@]+\.[^\s@]+$/u;

export function validateAuthInput(email: string, password: string): AuthValidationError | null {
  if (!email.trim() || !password) return 'required';
  if (!emailPattern.test(email.trim())) return 'invalid-email';
  if (Array.from(password).length < 6) return 'password-short';
  return null;
}

export function authEmailForSubmission(email: string): string {
  return email.trim();
}

export function passwordAutocomplete(mode: AuthMode): 'current-password' | 'new-password' {
  return mode === 'sign-in' ? 'current-password' : 'new-password';
}
