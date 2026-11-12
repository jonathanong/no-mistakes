declare function query(sql: string, values: unknown[]): Promise<{ rows: unknown[]; rowCount: number }>;
declare function checkpoint(query: unknown, id: unknown): Promise<void>;
declare function condition(): boolean;
declare const flag: boolean;
declare const other: boolean;
declare const helper: { go(query: unknown): void };

export async function shapes() {
  const stringLiteral = [{ run: () => query('stringLiteral', []) }];
  for (const stage of stringLiteral) {
    const result = await stage.run();
    if ('x') return;
  }
  const nullLiteral = [{ run: () => query('nullLiteral', []) }];
  for (const stage of nullLiteral) {
    const result = await stage.run();
    if (null) return;
  }
  const booleanLiteral = [{ run: () => query('booleanLiteral', []) }];
  for (const stage of booleanLiteral) {
    const result = await stage.run();
    if (true) return;
  }
  const staticMember = [{ run: () => query('staticMember', []) }];
  for (const stage of staticMember) {
    const result = await stage.run();
    if (result.rows) return;
  }
  const computedMember = [{ run: () => query('computedMember', []) }];
  for (const stage of computedMember) {
    const result = await stage.run();
    if (result.rows[0]) return;
  }
  const logical = [{ run: () => query('logical', []) }];
  for (const stage of logical) {
    const result = await stage.run();
    if (flag && other) return;
  }
  const binary = [{ run: () => query('binary', []) }];
  for (const stage of binary) {
    const result = await stage.run();
    if (result.rowCount > 0) return;
  }
  const chainStatic = [{ run: () => query('chainStatic', []) }];
  for (const stage of chainStatic) {
    const result = await stage.run();
    if (result?.rows) return;
  }
  const chainComputed = [{ run: () => query('chainComputed', []) }];
  for (const stage of chainComputed) {
    const result = await stage.run();
    if (result.rows?.[0]) return;
  }
  const objectReturn = [{ run: () => query('objectReturn', []) }];
  for (const stage of objectReturn) {
    const result = await stage.run();
    if (flag) return { done: flag };
  }
  const blockCheckpoint = [{ run: () => query('blockCheckpoint', []) }];
  for (const stage of blockCheckpoint) {
    const result = await stage.run();
    if (flag) { checkpoint(query, 'id'); return; }
  }
  const elseBranch = [{ run: () => query('elseBranch', []) }];
  for (const stage of elseBranch) {
    const result = await stage.run();
    if (flag) return; else other;
  }
  const bareConsequent = [{ run: () => query('bareConsequent', []) }];
  for (const stage of bareConsequent) {
    const result = await stage.run();
    if (flag) other;
  }
  const blockWithoutReturn = [{ run: () => query('blockWithoutReturn', []) }];
  for (const stage of blockWithoutReturn) {
    const result = await stage.run();
    if (flag) { checkpoint(query, 'id'); }
  }
  const blockPlainStatement = [{ run: () => query('blockPlainStatement', []) }];
  for (const stage of blockPlainStatement) {
    const result = await stage.run();
    if (flag) { other; return; }
  }
  const memberCheckpoint = [{ run: () => query('memberCheckpoint', []) }];
  for (const stage of memberCheckpoint) {
    const result = await stage.run();
    if (flag) { helper.go(query); return; }
  }
  const stageCheckpoint = [{ run: () => query('stageCheckpoint', []) }];
  for (const stage of stageCheckpoint) {
    const result = await stage.run();
    if (flag) { stage(query); return; }
  }
  const evalCheckpoint = [{ run: () => query('evalCheckpoint', []) }];
  for (const stage of evalCheckpoint) {
    const result = await stage.run();
    if (flag) { eval('x'); return; }
  }
  const stageArgument = [{ run: () => query('stageArgument', []) }];
  for (const stage of stageArgument) {
    const result = await stage.run();
    if (flag) { checkpoint(stage); return; }
  }
  const chainCall = [{ run: () => query('chainCall', []) }];
  for (const stage of chainCall) {
    const result = await stage.run();
    if (result?.go()) return;
  }
  const objectComputed = [{ run: () => query('objectComputed', []) }];
  for (const stage of objectComputed) {
    const result = await stage.run();
    if (flag) return { [other]: flag };
  }
  const objectMethod = [{ run: () => query('objectMethod', []) }];
  for (const stage of objectMethod) {
    const result = await stage.run();
    if (flag) return { go() {} };
  }
  const callCondition = [{ run: () => query('callCondition', []) }];
  for (const stage of callCondition) {
    const result = await stage.run();
    if (condition()) return;
  }
}
