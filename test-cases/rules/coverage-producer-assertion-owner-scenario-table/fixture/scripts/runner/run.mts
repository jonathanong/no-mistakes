import { equal } from 'node:assert/strict';
import { opaque } from '../../scenarios/opaque.mts';
import { gracefulShutdown } from '@fixture/shutdown';
import { runX, withoutAssertion } from '../../scenarios/run-x.mts';

function assertPlanRegistry(plans: readonly unknown[]) {
  if (plans.length === 0) throw new Error('no scenarios');
}
function assertNoScenarioFailures(errors: unknown[]) {
  if (errors.length > 0) throw new AggregateError(errors);
}
async function main() {
  const scenarios = [{ name: 'run-x', run: runX }] as const;
  const errors: unknown[] = [];
  for (const scenario of scenarios) {
    try { await scenario.run(); } catch (error) { errors.push(error); }
  }
  assertPlanRegistry(scenarios);
  assertNoScenarioFailures(errors);
}
export async function run() { await main().finally(gracefulShutdown); }

export async function directObject() {
  const scenarios = { runX };
  await scenarios.runX();
  await gracefulShutdown();
}
export async function noOwner() {
  const scenarios = [{ run: withoutAssertion }] as const;
  for (const scenario of scenarios) await scenario.run();
  await gracefulShutdown();
}

export async function inlineTable() {
  for (const scenario of [{ run: runX }]) await scenario.run();
  await gracefulShutdown();
}

export async function mixedFirst() {
  const table = [{ run: opaque }, { run: runX }];
  for (const row of table) await row.run();
  await gracefulShutdown();
}
export async function mixedLast() {
  const table = [{ run: runX }, { run: opaque }];
  for (const row of table) await row.run();
  await gracefulShutdown();
}
export async function directModuleOwner() {
  const table = [{ run: equal }];
  for (const row of table) await row.run();
  await gracefulShutdown();
}
