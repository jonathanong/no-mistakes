// An erased this parameter must never shift the runtime callback argument index.
export async function consume(this: unknown, callback: () => Promise<void>) { await callback(); }
