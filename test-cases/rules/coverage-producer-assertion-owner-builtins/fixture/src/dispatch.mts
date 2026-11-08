import { ok } from 'node:assert/strict';
import { owner } from './owner.mjs';

interface Sink { push(value: number): void }
function makeSink(): Sink { return { push(value) { ok(value > 0); } }; }
const sinks: readonly Sink[] = [makeSink()];
export function pushSink(): void {
  const sink = sinks[0];
  if (!sink) throw new Error('missing sink');
  sink.push(1);
}

interface Recorder { record(value: number): void }
function makeRecorder(): Recorder { return { record(value) { ok(value > 0); } }; }
const recorders: readonly Recorder[] = [makeRecorder()];
export function recordSink(): void {
  const recorder = recorders[0];
  if (!recorder) throw new Error('missing recorder');
  recorder.record(1);
}

export function replaceString(firstPath: string) { firstPath.replace(/re/, '/repeat-$1.json'); owner(); }
export function replaceInline(firstPath: string) { firstPath.replace(/re/, () => { owner(); return ''; }); }
export function replaceUnresolved(firstPath: string, fn: () => string) { firstPath.replace(/re/, fn); }
