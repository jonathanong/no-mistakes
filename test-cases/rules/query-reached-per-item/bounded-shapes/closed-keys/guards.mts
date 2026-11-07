declare function read(...args: any[]): Promise<any>;
type WrappedKind = ('post' | 'comment');
export async function parenthesizedKeys() {
  const grouped: Record<('post' | 'comment'), string[]> = {} as any;
  await Promise.all(Object.entries(grouped).map(([kind, ids]) => {
    return read(kind, ids);
  }));
}
export async function parenthesizedAliasKeys() {
  const grouped: Record<WrappedKind, string[]> = {} as any;
  await Promise.all(Object.entries(grouped).map(function ([kind, ids]) {
    return read(kind, ids);
  }));
}
export async function controlThirdParameter() {
  const grouped: Record<'post' | 'comment', string[]> = {} as any;
  await Promise.all(Object.entries(grouped).map(([kind, ids], index, source) => read(kind, ids)));
}
export async function controlRestParameter() {
  const grouped: Record<'post' | 'comment', string[]> = {} as any;
  await Promise.all(Object.entries(grouped).map((...entries) => read(entries[0])));
}
declare namespace Remote {
  type Kind = string;
  const keys: readonly string[];
}
export async function controlQualifiedType() {
  const grouped: Record<Remote.Kind, string[]> = {} as any;
  await Promise.all(Object.entries(grouped).map(([kind, ids]) => read(kind, ids)));
}
export async function controlIndexedArrayType() {
  const grouped: Record<string[][number], string[]> = {} as any;
  await Promise.all(Object.entries(grouped).map(([kind, ids]) => read(kind, ids)));
}
export async function controlQualifiedTypeQuery() {
  const grouped: Record<typeof Remote.keys[number], string[]> = {} as any;
  await Promise.all(Object.entries(grouped).map(([kind, ids]) => read(kind, ids)));
}
export async function controlFunctionThirdParameter() {
  const grouped: Record<'post' | 'comment', string[]> = {} as any;
  await Promise.all(Object.entries(grouped).map(function ([kind, ids], index, source) {
    return read(kind, ids);
  }));
}
export async function controlFunctionRestParameter() {
  const grouped: Record<'post' | 'comment', string[]> = {} as any;
  await Promise.all(Object.entries(grouped).map(function (...entries) {
    return read(entries[0]);
  }));
}
