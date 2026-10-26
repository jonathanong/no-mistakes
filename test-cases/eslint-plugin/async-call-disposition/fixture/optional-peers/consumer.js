export async function consume(callback) { await callback(); }
export async function consumeAgain(callback) { await callback(); }
export async function consumeProperties(dependencies) { await dependencies.enqueueEmail(); }
