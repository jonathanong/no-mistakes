if ((import.meta.main as boolean)!) { await run(); }
export const value = (await load()) satisfies unknown;
export async function run(): Promise<void> { await load(); }
