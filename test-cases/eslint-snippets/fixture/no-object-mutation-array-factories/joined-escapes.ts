declare function consume(value: unknown): void;
export function fresh(choose: boolean) {
  const left: number[] = [1];
  const right: number[] = [2];
  consume(right);
  const rows = choose ? left : right;
  const ready: number[] = [];
  rows.map(value => { ready.push(value); return value; });
}
export function shared(left: number[], right: number[], choose: boolean) {
  const rows = choose ? left : right;
  consume(right);
  const ready: number[] = [];
  rows.map(value => { ready.push(value); return value; });
}
export function overridden(left: number[], right: number[], choose: boolean) {
  const rows = choose ? left : right;
  rows.map = callback => [callback(1)];
  const ready: number[] = [];
  right.map(value => { ready.push(value); return value; });
}
declare function getRows(): number[];
export function packed() {
  const left = getRows();
  const right = getRows();
  const groups = [left, right];
  consume(right);
  const ready: number[] = [];
  groups[0].forEach(value => { ready.push(value); });
}
