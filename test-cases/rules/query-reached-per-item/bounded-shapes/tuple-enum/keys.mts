declare function read(...args: any[]): Promise<any>;
enum Kind { Post = 'post', Comment = 'comment' }
export async function enumKeys() {
  for (const kind in Kind) await read(kind);
}
export async function controlEnumKeys(Kind: Record<string, string>) {
  for (const kind in Kind) await read(kind);
}
