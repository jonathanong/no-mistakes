import * as proc from 'node:process';
import { opaque } from '../../scenarios/opaque.mts';
import { gracefulShutdown } from '@fixture/shutdown';
import { mutate } from 'fixture-runtime';
import { runX, withoutAssertion, throwsWithoutAssertion } from '../../scenarios/run-x.mts';

export async function dynamic(table: any) {
  for (const scenario of table) await scenario.run();
  await gracefulShutdown();
}
export async function mutated() {
  const table = [{ run: runX }];
  table[0].run = withoutAssertion;
  for (const scenario of table) await scenario.run();
  await gracefulShutdown();
}
export async function escaped() {
  const table = [{ run: runX }];
  mutate(table);
  for (const scenario of table) await scenario.run();
  await gracefulShutdown();
}
export async function empty() {
  const table: any[] = [];
  for (const scenario of table) await scenario.run();
  await gracefulShutdown();
}
export async function uninvoked() {
  const table = [{ run: runX }];
  void table;
  await gracefulShutdown();
}
export async function firstDestructured() {
  const [callback] = [withoutAssertion, runX];
  await callback();
  await gracefulShutdown();
}
export async function earlyBreak() {
  const table = [{ run: withoutAssertion }, { run: runX }] as const;
  for (const scenario of table) { await scenario.run(); break; }
  await gracefulShutdown();
}
export async function earlyReturn() {
  const table = [{ run: withoutAssertion }, { run: runX }] as const;
  for (const scenario of table) { await scenario.run(); return; }
}
export async function filtered() {
  const table = [{ enabled: true, run: withoutAssertion }, { enabled: false, run: runX }];
  for (const scenario of table) { if (scenario.enabled) await scenario.run(); }
  await gracefulShutdown();
}
export async function shadowed() {
  const table = [{ run: runX }];
  for (const scenario of table) {
    { const scenario = { run: withoutAssertion }; await scenario.run(); }
  }
  await gracefulShutdown();
}
export async function overwrittenRow() {
  const table = [{ run: runX }];
  for (const scenario of table) { scenario.run = withoutAssertion; await scenario.run(); }
  await gracefulShutdown();
}

const GLOBAL_TABLE = [{ run: runX }];
export async function globalTable() {
  for (const scenario of GLOBAL_TABLE) await scenario.run();
  await gracefulShutdown();
}
export async function aliasedTable() {
  const table = [{ run: runX }];
  const alias = table;
  for (const scenario of alias) await scenario.run();
  await gracefulShutdown();
}
export async function malformedRow() {
  for (const scenario of [null, { run: runX }]) await scenario.run();
  await gracefulShutdown();
}

const MUTATED_GLOBAL = [{ run: runX }];
export async function globalMutated() {
  for (const scenario of MUTATED_GLOBAL) await scenario.run();
  await gracefulShutdown();
}
MUTATED_GLOBAL[0].run = withoutAssertion;
const ESCAPED_GLOBAL = [{ run: runX }];
export async function globalEscaped() {
  for (const scenario of ESCAPED_GLOBAL) await scenario.run();
  await gracefulShutdown();
}
mutate(ESCAPED_GLOBAL);
export async function captured() {
  const table = [{ run: runX }];
  erase();
  for (const scenario of table) await scenario.run();
  function erase() { table[0].run = withoutAssertion; }
  await gracefulShutdown();
}

export async function earlierEscapeLoop() {
  const table = [{ run: runX }];
  for (const row of table) { mutate(row); }
  for (const row of table) { await row.run(); }
  await gracefulShutdown();
}
export async function earlierMutationLoop() {
  const table = [{ run: runX }];
  for (const row of table) { row.run = withoutAssertion; }
  for (const row of table) { await row.run(); }
  await gracefulShutdown();
}

export async function opaqueWithoutOwner() {
  const table = [{ run: opaque }, { run: withoutAssertion }];
  for (const row of table) await row.run();
  await gracefulShutdown();
}
export async function onlyOpaque() {
  const table = [{ run: opaque }];
  for (const row of table) await row.run();
  await gracefulShutdown();
}

export async function exitingCatch() {
  const table = [{ run: throwsWithoutAssertion }, { run: runX }];
  for (const row of table) {
    try { await row.run(); } catch { process.exit(); }
  }
  await gracefulShutdown();
}
export async function abortingCatch() {
  const table = [{ run: throwsWithoutAssertion }, { run: runX }];
  for (const row of table) {
    try { await row.run(); } catch { process.abort(); }
  }
  await gracefulShutdown();
}

export async function exitingCatchWithCode() {
  const table = [{ run: throwsWithoutAssertion }, { run: runX }];
  for (const row of table) {
    try { await row.run(); } catch { process.exit(1); }
  }
  await gracefulShutdown();
}

export async function aliasedExitingCatch() {
  const table = [{ run: throwsWithoutAssertion }, { run: runX }];
  for (const row of table) {
    try { await row.run(); } catch { proc.exit(); }
  }
  await gracefulShutdown();
}
