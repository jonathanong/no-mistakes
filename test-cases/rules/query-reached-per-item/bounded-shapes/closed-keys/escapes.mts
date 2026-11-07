declare function read(...args: any[]): Promise<any>;
declare class Mutator { constructor(grouped: unknown, items: unknown); }
declare function mutate(strings: TemplateStringsArray, ...values: unknown[]): void;
export async function controlConstructorEscape(items: Record<string, string[]>) {
  const grouped: Record<'post' | 'comment', string[]> = {} as any;
  new Mutator(grouped, items);
  await Promise.all(Object.entries(grouped).map(([kind, ids]) => read(kind, ids)));
}
export async function controlTaggedEscape(items: Record<string, string[]>) {
  const grouped: Record<'post' | 'comment', string[]> = {} as any;
  mutate`${grouped}${items}`;
  await Promise.all(Object.entries(grouped).map(([kind, ids]) => read(kind, ids)));
}
