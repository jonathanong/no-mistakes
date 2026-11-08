import DefaultSdk, { NamedSdk, NamespaceSdk } from './sdk-barrel.mjs';
import { ValueSdk } from './sdk.mjs';
import * as Barrel from './sdk-barrel.mjs';
import { NamespaceSdk as AmbiguousSdk } from './sdk-ambiguous.mjs';
import { NamespaceSdk as CyclicSdk } from './sdk-cycle-a.mjs';
import { ok } from 'node:assert/strict';
export function defaultNamespace(error: unknown) { DefaultSdk.captureException(error); }
export function namedNamespace(error: unknown) { NamedSdk.captureException(error); }
export function exportNamespace(error: unknown) { NamespaceSdk.captureException(error); }
export function shadowedNamespace(DefaultSdk: { captureException(error: unknown): void }, error: unknown) { DefaultSdk.captureException(error); }
export function mapGet() {
  const handlers = new Map<string, () => void>();
  handlers.set('check', () => ok(true));
  const handler = handlers.get('check');
  handler?.();
}
export function arrayCallback() {
  const checks: Array<() => void> = [() => ok(true)];
  checks.forEach(check => check());
}
export function nestedNamespace(error: unknown) { Barrel.NamedSdk.captureException(error); }
export function builtinNameOnly(receiver: { get(): () => void }) { receiver.get()(); }
export function ambiguousNamespace(error: unknown) { AmbiguousSdk.captureException(error); }
export function cyclicNamespace(error: unknown) { CyclicSdk.captureException(error); }
export function bareNamespace() { DefaultSdk(); }
export function externalValue(error: unknown) { ValueSdk.captureException(error); }
