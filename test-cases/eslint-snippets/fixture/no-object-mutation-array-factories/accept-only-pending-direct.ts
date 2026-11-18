type Ev = { name: string; target?: { id: string } };
export function direct(events: Ev[]) {
  const event = events[0];
  if (!event) return;
  event.target = { id: "example" };
}
