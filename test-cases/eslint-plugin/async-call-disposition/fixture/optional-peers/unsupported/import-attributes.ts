import data from './data.json' with { type: 'json' };
export async function consume(callback: () => Promise<void>) { await callback(); }
