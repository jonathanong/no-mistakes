export async function beginTransaction(): Promise<(sql: string) => unknown> {
  throw new Error('The configured factory is a static contract in this fixture');
}

export { beginTransaction as default };

export const unrelatedFactory = beginTransaction;
