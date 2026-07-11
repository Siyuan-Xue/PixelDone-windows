export async function invoke<T = any>(command: string, args?: Record<string, unknown>): Promise<T> {
  return browser.tauri.execute(
    ({ core }, input) => core.invoke(input.command, input.args),
    { command, args }
  ) as Promise<T>;
}

export async function bootstrap(): Promise<any> {
  return invoke('bootstrap');
}
