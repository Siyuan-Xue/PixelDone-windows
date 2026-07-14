import { describe, expect, test } from 'bun:test';
import {
  authEmailForSubmission,
  passwordAutocomplete,
  validateAuthInput
} from '../src/lib/components/auth/auth';

describe('authentication modal state', () => {
  test('validates required, email and password rules before IPC', () => {
    expect(validateAuthInput('', '')).toBe('required');
    expect(validateAuthInput('not-an-email', 'secret')).toBe('invalid-email');
    expect(validateAuthInput('person@example.com', 'short')).toBe('password-short');
    expect(validateAuthInput('person@example.com', 'secret')).toBeNull();
  });

  test('normalizes email and changes password autocomplete by mode', () => {
    expect(authEmailForSubmission('  person@example.com  ')).toBe('person@example.com');
    expect(passwordAutocomplete('sign-in')).toBe('current-password');
    expect(passwordAutocomplete('sign-up')).toBe('new-password');
  });
});
