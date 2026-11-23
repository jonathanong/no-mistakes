export interface LegacyAdapter {
  read(key: string): Promise<string | undefined>;
  write(key: string, value: string): Promise<void>;
}

export async function updateValue(
  adapter: LegacyAdapter,
  key: string,
  update: (current: string | undefined) => string,
): Promise<void> {
  const current = await adapter.read(key);
  await adapter.write(key, update(current));
}
