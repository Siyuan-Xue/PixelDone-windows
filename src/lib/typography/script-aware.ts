export type PixelTextRole = 'sans' | 'serif';

export type PixelTextScript = 'source' | 'cjk' | 'arabic' | 'system';

export interface PixelScriptRun {
  text: string;
  script: PixelTextScript;
}

interface ScriptToken {
  text: string;
  script: PixelTextScript | null;
}

const sourcePattern = /[\p{Script=Latin}\p{Script=Cyrillic}\p{Script=Greek}]/u;
const cjkPattern = /[\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}\p{Script=Hangul}]/u;
const arabicPattern = /\p{Script=Arabic}/u;
const emojiPattern = /(?:\p{Extended_Pictographic}|\p{Emoji_Modifier}|\p{Regional_Indicator}|[\u200d\ufe0e\ufe0f\u20e3])/u;

export function splitScriptRuns(text: string): PixelScriptRun[] {
  if (!text) return [];

  const tokens: ScriptToken[] = Array.from(text, (character) => ({
    text: character,
    script: scriptFor(character)
  }));
  const resolved = tokens.map((token, index) => token.script ?? nearestWrittenScript(tokens, index) ?? 'source');

  return tokens.reduce<PixelScriptRun[]>((runs, token, index) => {
    const script = resolved[index];
    const previous = runs.at(-1);
    if (previous?.script === script) {
      previous.text += token.text;
    } else {
      runs.push({ text: token.text, script });
    }
    return runs;
  }, []);
}

export function scriptRunClass(script: PixelTextScript, role: PixelTextRole): string {
  return `script-${script}-${role}`;
}

function nearestWrittenScript(tokens: ScriptToken[], index: number): PixelTextScript | null {
  for (let previous = index - 1; previous >= 0; previous -= 1) {
    const script = tokens[previous].script;
    if (script && script !== 'system') return script;
  }
  for (let next = index + 1; next < tokens.length; next += 1) {
    const script = tokens[next].script;
    if (script && script !== 'system') return script;
  }
  return null;
}

function scriptFor(character: string): PixelTextScript | null {
  if (emojiPattern.test(character)) return 'system';
  if (sourcePattern.test(character)) return 'source';
  if (cjkPattern.test(character)) return 'cjk';
  if (arabicPattern.test(character)) return 'arabic';
  return null;
}
