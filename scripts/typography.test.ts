import { describe, expect, test } from 'bun:test';
import { scriptRunClass, splitScriptRuns } from '../src/lib/typography/script-aware';

describe('script-aware typography', () => {
  test('maps declared scripts to deterministic font runs', () => {
    expect(splitScriptRuns('Work \u4e2d\u6587 \u0627\u0644\u0639\u0631\u0628\u064a\u0629 \u0420\u0443\u0441\u0441\u043a\u0438\u0439')).toEqual([
      { text: 'Work ', script: 'source' },
      { text: '\u4e2d\u6587 ', script: 'cjk' },
      { text: '\u0627\u0644\u0639\u0631\u0628\u064a\u0629 ', script: 'arabic' },
      { text: '\u0420\u0443\u0441\u0441\u043a\u0438\u0439', script: 'source' }
    ]);
  });

  test('attaches punctuation to the nearest written script and leaves emoji on the system font', () => {
    expect(splitScriptRuns('\u300c\u4e2d\u6587\u300d\u{1f4cc}Test')).toEqual([
      { text: '\u300c\u4e2d\u6587\u300d', script: 'cjk' },
      { text: '\u{1f4cc}', script: 'system' },
      { text: 'Test', script: 'source' }
    ]);
    expect(splitScriptRuns('A\u{1f468}\u200d\u{1f469}\u200d\u{1f467}\u200d\u{1f466}\u4e2d')).toEqual([
      { text: 'A', script: 'source' },
      { text: '\u{1f468}\u200d\u{1f469}\u200d\u{1f467}\u200d\u{1f466}', script: 'system' },
      { text: '\u4e2d', script: 'cjk' }
    ]);
  });

  test('exposes stable role-specific CSS classes', () => {
    expect(scriptRunClass('arabic', 'serif')).toBe('script-arabic-serif');
    expect(scriptRunClass('cjk', 'sans')).toBe('script-cjk-sans');
  });
});
