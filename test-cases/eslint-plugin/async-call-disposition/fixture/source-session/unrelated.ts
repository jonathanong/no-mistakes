import { enqueue } from '@session/jobs';

export function summarize(rows: string[], measure: (value: string) => number, count: number) {
  rows.map((row) => row.trim()).reduce((total, row) => total + row.length, 0);
  measure('sample');
  count.toFixed();
  // The source model cannot resolve member factories or IIFEs as declarations.
  const holder = { factory() { return { run: enqueue }; } };
  holder.factory().run('member');
  (() => ({ run: enqueue }))().run('iife');
}
