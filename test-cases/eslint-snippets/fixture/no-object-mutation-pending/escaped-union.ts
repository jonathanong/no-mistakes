type Ev = { name: string; target?: { id: string } };

function leak(value: unknown) {
  return value;
}

export function kept() {
  const pending: Array<{ event: Ev }> = [];
  pending.push({ event: { name: "one" } });
  pending.push({ event: { name: "two" } });
  pending[1]!.event.target = { id: "x" };
}

export function tainted() {
  const pending: Array<{ event: Ev }> = [];
  pending.push({ event: { name: "one" } });
  pending.push({ event: { name: "two" } });
  leak(pending[0]);
  pending[1]!.event.target = { id: "x" };
}
