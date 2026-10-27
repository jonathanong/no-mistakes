enum State { Ready }
export async function consume(callback: () => Promise<void>) { await callback(); }
