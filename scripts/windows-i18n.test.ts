import { describe, expect, it } from 'bun:test';
import { windowsMessage } from '../src/lib/i18n/windows';

const signedOutSyncMessages = {
  en: 'Sign in to sync with Android.',
  'zh-Hans': '登录后可与 Android 同步。',
  ar: 'سجّل الدخول للمزامنة مع Android.',
  fr: 'Connectez-vous pour synchroniser avec Android.',
  ru: 'Войдите, чтобы синхронизироваться с Android.',
  es: 'Inicia sesión para sincronizar con Android.'
} as const;

describe('Windows localization', () => {
  for (const [locale, expected] of Object.entries(signedOutSyncMessages)) {
    it(`localizes the signed-out Android sync hint for ${locale}`, () => {
      expect(windowsMessage(locale as keyof typeof signedOutSyncMessages, 'signInToSyncAndroid')).toBe(expected);
    });
  }
});
