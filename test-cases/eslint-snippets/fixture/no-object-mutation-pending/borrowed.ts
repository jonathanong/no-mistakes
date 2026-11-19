type Ev = { name: string; target?: { id: string } };
type Entry = { args: Record<string, unknown>; event: Ev };
export function loop(bodies: unknown[], events: Ev[]) {
  const pending: Entry[] = [];
  for (const [i, body] of bodies.entries()) {
    const event = events[i]!;
    pending.push({ args: body as Record<string, unknown>, event });
  }
  for (const { args, event } of pending) {
    args["id"] = 1;
    event.target = { id: "x" };
  }
}
export function twoPushes(events: Ev[]) {
  const pending: Array<{ event: Ev }> = [];
  pending.push({ event: events[0]! });
  pending.push({ event: events[1]! });
  const { event } = pending[0]!;
  event.target = { id: "x" };
}
export function mixed(events: Ev[]) {
  const pending: Array<{ event: Ev }> = [];
  pending.push({ event: { name: "fresh" } });
  pending.push({ event: events[0]! });
  pending.push({ event: events[0]! });
  const { event } = pending[1]!;
  event.target = { id: "x" };
}
export function values(bodies: unknown[]) {
  for (const body of bodies.values()) {
    (body as Record<string, unknown>)["id"] = 1;
  }
}
