export function recordItems(items: string[]): void {
  const seen = new Set<string>();
  const lookup = new Map<string, string>() as unknown as { add(v: string): void };
  const weak = new WeakSet<object>() as unknown as { add(v: object): void };
  const bucket = [] as unknown as { add(v: string): void };
  for (const item of items) {
    seen.add(item);
  }
  lookup.add("a");
  weak.add({});
  bucket.add("b");
}
