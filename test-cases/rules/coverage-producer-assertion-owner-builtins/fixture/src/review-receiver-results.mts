import { owner } from './owner.mjs';

export const SHARDS = ['a', 'b'] as const;

export function shards() {
  const joined = SHARDS.join(',');
  return SHARDS.find((label) => label.startsWith('a') && joined.length > 0);
}

export function isoStamp() {
  return new Date().toISOString().replace(/Z$/, '000Z');
}

export function boundDate() {
  const date = new Date();
  return date.toISOString().replace(/Z$/, '000Z');
}

async function main(): Promise<void> {
  return;
}

export function asyncShutdown() {
  return main().finally(() => undefined);
}

interface Row {
  scenario: string;
}

function getResults(): Row[] {
  return [{ scenario: 'a' }];
}

export function localResults() {
  return getResults().map((result) => result.scenario);
}

export function boundRows() {
  const resultRows = getResults();
  return resultRows.map((row) => row.scenario);
}

interface Failure {
  scenario: string;
}

const failures: Failure[] = [];

export function recordFailure(scenario: string): void {
  failures.push({ scenario });
}

export function moduleFailures() {
  return failures.map((failure) => failure.scenario);
}

export function localRows() {
  const rows = ['a', 'b'];
  return rows.map((row) => row.length);
}

export function untypedMap(values) {
  return values.map((row) => row);
}

export function unprovenCallback(fn: () => void) {
  fn();
}

function run(fn: () => void) {
  fn();
}

export function provenCallback() {
  run(() => owner());
}
