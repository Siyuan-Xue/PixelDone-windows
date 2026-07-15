export type PasswordValidationError =
  | 'required'
  | 'password-short'
  | 'same-password'
  | 'confirmation-mismatch';

export function validatePasswordChange(
  currentPassword: string,
  newPassword: string,
  confirmation: string
): PasswordValidationError | null {
  if (!currentPassword || !newPassword || !confirmation) return 'required';
  if (Array.from(newPassword).length < 6) return 'password-short';
  if (currentPassword === newPassword) return 'same-password';
  if (newPassword !== confirmation) return 'confirmation-mismatch';
  return null;
}
