class State { value = 1; }
export async function consume(callback: () => Promise<void>) { await callback(); }
