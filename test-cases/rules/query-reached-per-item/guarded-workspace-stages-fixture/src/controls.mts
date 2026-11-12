import type { TransactionQuery } from '@pkg/psql';
declare const flag: boolean;
declare function checkpoint(query: TransactionQuery, id: unknown): Promise<void>;
declare function condition(): boolean;
declare function escape(value: unknown): void;
declare function effect(): unknown;
export async function controls(query: TransactionQuery) {
  const conditionCall = [{ run: () => query('condition call', []) }];
  for (const stage of conditionCall) {
    const result = await stage.run();
    if (condition()) return { done: false };
  }
  const conditionMutation = [{ run: () => query('condition mutation', []) }];
  for (const stage of conditionMutation) {
    const result = await stage.run();
    if ((stage.run = async () => ({ rows: [], rowCount: 0 }))) return;
  }
  const fallthrough = [{ run: () => query('fallthrough', []) }];
  for (const stage of fallthrough) {
    const result = await stage.run();
    if (flag) { await checkpoint(query, 'id'); }
  }
  const stageArgument = [{ run: () => query('stage argument', []) }];
  for (const stage of stageArgument) {
    const result = await stage.run();
    if (flag) { escape(stage); return; }
  }
  const sourceArgument = [{ run: () => query('source argument', []) }];
  for (const stage of sourceArgument) {
    const result = await stage.run();
    if (flag) { escape(sourceArgument); return; }
  }
  const effectfulReturn = [{ run: () => query('effectful return', []) }];
  for (const stage of effectfulReturn) {
    const result = await stage.run();
    if (flag) return effect();
  }
}
