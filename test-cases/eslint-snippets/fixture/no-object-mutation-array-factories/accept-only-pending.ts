type Ev = { name: string; target?: { id: string } };
declare function lookup(keys: string[]): Promise<Array<{ id: string } | undefined>>;
export async function prepare(bodies: unknown[], events: Ev[]) {
  const pending: Array<{ args: Record<string, unknown>; event: Ev; key: string }> = [];
  for (const [i, body] of bodies.entries()) {
    const event = events[i];
    if (!event || typeof body !== 'object' || body === null) continue;
    pending.push({ args: body as Record<string, unknown>, event, key: event.name });
  }
  const found = await lookup(pending.map(p => p.key));
  for (const [i, hit] of found.entries()) {
    if (!hit) continue;
    const { args, event } = pending[i]!;
    args['id'] = hit.id;
    event.target = { id: hit.id };
  }
}
