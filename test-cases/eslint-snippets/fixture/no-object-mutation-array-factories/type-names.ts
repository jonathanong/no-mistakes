type Object = { fake: true };
type Array<T> = ReadonlyArray<T>;
export function nativeValues(groups: Record<string, number>) {
  const ready: number[] = [];
  Object.values(groups).forEach(value => { ready.push(value); });
  Array.from([1]).forEach(value => { ready.push(value); });
  return ready;
}
