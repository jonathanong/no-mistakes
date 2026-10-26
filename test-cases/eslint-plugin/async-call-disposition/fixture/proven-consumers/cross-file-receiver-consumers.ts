export async function consume(deps: any) { await deps.other(); await deps.job(); }
export async function consumeAgain(deps: any) { await deps.other(); await deps.job(); }
export async function safeImported() { const holes = [,]; return holes.length; }
