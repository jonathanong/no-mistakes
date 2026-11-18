export function parameters(left: number[], right: number[], choose: boolean) {
  const rows = choose ? left : right;
  const ready: number[] = [];
  rows.map(value => { ready.push(value); return value; });
  return ready;
}
export function literals(choose: boolean) {
  const rows = choose ? [1] : [2];
  const ready: number[] = [];
  rows.forEach(value => { ready.push(value); });
  return ready;
}
declare function getRows(): number[];
export function opaque(choose: boolean) {
  const left = getRows();
  const right = getRows();
  const rows = choose ? left : right;
  const ready: number[] = [];
  rows.map(value => { ready.push(value); return value; });
  return ready;
}
