import { gracefulShutdown } from '@fixture/shutdown';
import { runX, throwsWithoutAssertion, withArgument } from '../../scenarios/run-x.mts';

export async function excessiveBody() {
  const table = [{ run: runX }];
  for (const row of table) {
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
    await row.run();
  }
  await gracefulShutdown();
}
export async function nestedReceiver() {
  const table = [{ handlers: { run: runX } }];
  for (const row of table) await row.handlers.run();
  await gracefulShutdown();
}
export async function sourceReceiver() {
  const table = [{ run: runX }];
  for (const row of table) table.pop();
  await gracefulShutdown();
}
export async function rowArgument() {
  const table = [{ run: withArgument }];
  for (const row of table) await row.run('scenario input');
  await gracefulShutdown();
}
export async function unrelatedBody() {
  const table = [{ run: runX }];
  for (const row of table) console.log('scenario skipped');
  await gracefulShutdown();
}
export async function structuredCatch() {
  const table = [{ run: throwsWithoutAssertion }, { run: runX }];
  const errors: unknown[] = [];
  for (const row of table) {
    try { await row.run(); } catch (error) { errors.push({ error }); }
  }
  await gracefulShutdown();
}
function scenarioFactory() { return [{ run: runX }]; }
export async function factoryTable() {
  for (const row of scenarioFactory()) await row.run();
  await gracefulShutdown();
}
export async function nestedMetadata() {
  const table = [{ run: runX, metadata: { name: 'scenario' } }];
  for (const row of table) await row.run();
  await gracefulShutdown();
}
export async function localCallback() {
  const callback = runX;
  const table = [{ run: callback }];
  for (const row of table) await row.run();
  await gracefulShutdown();
}
export async function literalCatch() {
  const table = [{ run: throwsWithoutAssertion }, { run: runX }];
  const errors: unknown[] = [];
  for (const row of table) {
    try { await row.run(); } catch { errors.push('scenario failed'); }
  }
  await gracefulShutdown();
}
