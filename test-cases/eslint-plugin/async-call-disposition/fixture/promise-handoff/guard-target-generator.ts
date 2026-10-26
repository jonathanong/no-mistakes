// Async generators produce iterators rather than a promise to adopt at the call.
export async function* enqueueEmail(entries: unknown, options: unknown) { void entries; yield options; }
