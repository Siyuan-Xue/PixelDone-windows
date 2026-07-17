export async function invokeRaw<T = any>(command: string, args?: Record<string, unknown>): Promise<T> {
  return browser.tauri.execute(
    ({ core }, input) => core.invoke(input.command, input.args),
    { command, args }
  ) as Promise<T>;
}

function isStaleRevision(error: unknown): boolean {
  const message = error instanceof Error ? error.message : String(error);
  return message.includes('STALE_REVISION') || message.includes('当前界面状态已过期');
}

export async function invoke<T = any>(command: string, args?: Record<string, unknown>): Promise<T> {
  let currentArgs = args;
  for (let attempt = 0; attempt < 3; attempt += 1) {
    try {
      return await invokeRaw<T>(command, currentArgs);
    } catch (error) {
      if (
        attempt === 2 ||
        typeof currentArgs?.expectedRevision !== 'number' ||
        !isStaleRevision(error)
      ) {
        throw error;
      }
      const snapshot = await invokeRaw<any>('bootstrap');
      currentArgs = { ...currentArgs, expectedRevision: snapshot.revision };
    }
  }
  throw new Error(`Unable to invoke ${command}`);
}

export async function bootstrap(): Promise<any> {
  return invoke('bootstrap');
}
