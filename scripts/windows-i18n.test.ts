import { describe, expect, it } from 'bun:test';
import { windowsMessage } from '../src/lib/i18n/windows';
import { uiText, uiWindowsMessage } from '../src/lib/i18n/presentation';

const signedOutSyncMessages = {
  en: 'Sign in to sync with Android',
  'zh-Hans': '登录后可与 Android 同步',
  ar: 'سجّل الدخول للمزامنة مع Android',
  fr: 'Connectez-vous pour synchroniser avec Android',
  ru: 'Войдите, чтобы синхронизироваться с Android',
  es: 'Inicia sesión para sincronizar con Android'
} as const;

describe('Windows localization', () => {
  for (const [locale, expected] of Object.entries(signedOutSyncMessages)) {
    it(`localizes the signed-out Android sync hint for ${locale}`, () => {
      expect(uiWindowsMessage(locale as keyof typeof signedOutSyncMessages, 'signInToSyncAndroid')).toBe(expected);
    });
  }

  it('provides password-change labels for every supported locale', () => {
    for (const locale of Object.keys(signedOutSyncMessages)) {
      expect(windowsMessage(locale as keyof typeof signedOutSyncMessages, 'changePassword').trim()).not.toBe('');
      expect(windowsMessage(locale as keyof typeof signedOutSyncMessages, 'currentPassword').trim()).not.toBe('');
      expect(windowsMessage(locale as keyof typeof signedOutSyncMessages, 'confirmPassword').trim()).not.toBe('');
    }
  });

  it('removes terminal periods from UI copy without changing other punctuation', () => {
    expect(uiText('Ready.')).toBe('Ready');
    expect(uiText('准备就绪。')).toBe('准备就绪');
    expect(uiText('جاهز۔')).toBe('جاهز');
    expect(uiText('Delete the database?')).toBe('Delete the database?');
  });
});
