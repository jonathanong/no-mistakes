import { strictEqual } from 'node:assert';
import * as logger from 'fixture-logger';
import * as Sentry from '@sentry/node';
import { gracefulShutdown } from './registry.mts';

function assertNoFailures(errors: unknown[]) {
  strictEqual(errors.length, 0);
}

export function loggerOnly() { logger.info('coverage started'); }
export function mapOnly(rows: any[]) { return rows.map(row => row.execute()); }
export function flatMapOnly(rows: any[]) { return rows.flatMap(row => row.execute()); }
export function sentryOnly() { Sentry.captureException(new Error('failure')); }
export function cleanupOnly() { return gracefulShutdown(); }

async function main() {
  loggerOnly();
  mapOnly(executableRows());
  flatMapOnly(executableRows());
  sentryOnly();
  assertNoFailures([]);
}

async function mainWithoutAssertion() {
  loggerOnly();
  mapOnly(executableRows());
  flatMapOnly(executableRows());
  sentryOnly();
}

export function run() { return main().finally(gracefulShutdown); }
export function withoutAssertion() { return mainWithoutAssertion().finally(gracefulShutdown); }
export function loggerWithOwner() { loggerOnly(); assertNoFailures([]); }
export function mapWithOwner() { mapOnly(executableRows()); assertNoFailures([]); }
export function flatMapWithOwner() { flatMapOnly(executableRows()); assertNoFailures([]); }
export function sentryWithOwner() { sentryOnly(); assertNoFailures([]); }
export function cleanupWithOwner() { gracefulShutdown(); assertNoFailures([]); }
export function noOwner() { return 1; }

function executableRows(): any[] {
  return [{ execute() { logger.info('row executed'); return []; } }];
}
