import { message, type Locale, type MessageKey } from '$lib/generated/i18n';
import {
  windowsAuthValidationMessage,
  windowsReliabilityMessage,
  windowsMessage,
  type WindowsAuthValidationKey,
  type WindowsReliabilityMessageKey,
  type WindowsMessageKey
} from '$lib/i18n/windows';

const terminalPeriodPattern = /[.。۔]+(?=\s*$)/u;

export function uiText(value: string): string {
  return value.replace(terminalPeriodPattern, '');
}

export function uiMessage(locale: Locale, key: MessageKey): string {
  return uiText(message(locale, key));
}

export function uiWindowsMessage(locale: Locale, key: WindowsMessageKey): string {
  return uiText(windowsMessage(locale, key));
}

export function uiWindowsAuthValidationMessage(
  locale: Locale,
  key: WindowsAuthValidationKey
): string {
  return uiText(windowsAuthValidationMessage(locale, key));
}

export function uiWindowsReliabilityMessage(
  locale: Locale,
  key: WindowsReliabilityMessageKey
): string {
  return uiText(windowsReliabilityMessage(locale, key));
}
