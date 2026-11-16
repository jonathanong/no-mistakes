export async function bindingFlows(kind: 'orders' | 'refunds') {
  const awaited = await kind;
  const nested = `${await `${kind}`}`;
  const literal = 'orders';
  let mutable = kind;
  const recursive = recursive;
  const widenedAwaited = awaited as string;
  const widenedNested = nested as string;
  const widenedLiteral = literal as string;
  const widenedMutable = mutable as string;
  const widenedRecursive = recursive as string;
  return [widenedAwaited, widenedNested, widenedLiteral, widenedMutable, widenedRecursive];
}
