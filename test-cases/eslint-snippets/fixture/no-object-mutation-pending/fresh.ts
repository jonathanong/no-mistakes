type Ev = { name: string; target?: { id: string } };
type Entry = { args: Record<string, unknown>; event: Ev };
export function created(bodies: unknown[]) {
  const pending: Entry[] = [];
  for (const [i] of bodies.entries()) {
    const event: Ev = { name: String(i) };
    pending.push({ args: {}, event });
  }
  for (const [i, hit] of pending.entries()) {
    hit.args["id"] = i;
    hit.event.target = { id: String(i) };
  }
}
export function mapped(names: string[]) {
  const pending = names.map((name) => ({ args: {} as Record<string, unknown>, event: { name } }));
  for (const { args, event } of pending) {
    args["id"] = 1;
    (event as Ev).target = { id: "x" };
  }
}
export function twice() {
  const pending: Entry[] = [];
  pending.push({ args: {}, event: { name: "a" } });
  pending.push({ args: {}, event: { name: "b" } });
  const { args, event } = pending[0]!;
  args["id"] = 1;
  event.target = { id: "x" };
}
export function copy(rows: unknown[]) {
  return rows.slice();
}
