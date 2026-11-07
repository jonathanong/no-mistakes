declare function read(...args: any[]): Promise<any>;
declare global {
  interface ReadonlyArray<T> {
    mutate(strings: TemplateStringsArray, ...values: unknown[]): void;
  }
}
export async function controlTaggedTupleReceiver(items: string[]) {
  const kinds = ['post', 'comment'] as const;
  kinds.mutate`${items}`;
  for (const kind of kinds) await read(kind);
}
