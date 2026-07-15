import { describe, expect, test } from 'bun:test';
import { validatePasswordChange } from '../src/lib/components/auth/password';

describe('password modal state', () => {
  test('validates every password rule before IPC', () => {
    expect(validatePasswordChange('', '', '')).toBe('required');
    expect(validatePasswordChange('old-secret', 'short', 'short')).toBe('password-short');
    expect(validatePasswordChange('same-secret', 'same-secret', 'same-secret')).toBe('same-password');
    expect(validatePasswordChange('old-secret', 'new-secret', 'different')).toBe('confirmation-mismatch');
    expect(validatePasswordChange('old-secret', 'new-secret', 'new-secret')).toBeNull();
  });

  test('counts Unicode characters rather than UTF-8 bytes', () => {
    expect(validatePasswordChange('old-secret', '\u4e00\u4e8c\u4e09\u56db\u4e94', '\u4e00\u4e8c\u4e09\u56db\u4e94')).toBe('password-short');
    expect(validatePasswordChange('old-secret', '\u4e00\u4e8c\u4e09\u56db\u4e94\u516d', '\u4e00\u4e8c\u4e09\u56db\u4e94\u516d')).toBeNull();
  });
});
