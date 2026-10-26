import { enqueueEmail } from "@app/jobs";
import type { Dependencies, DeclaredDependencies, AmbientDependencies, SynchronousDependencies, ShadowedDependencies, AliasedDependencies } from './types';
import { prepareCrawl, prepareDropped } from './handler';
import { untypedFinal } from './untyped-final';

// The selected property is copied inside a handler that this module only returns.
// Adoption is not visible to the cross-file copy proof; a typed target slot is.
export function enqueueReferralLinkCrawlsForUrlId(dependencies: Dependencies) {
  const urlId = dependencies.urlId;
  return prepareCrawl({ ...dependencies, urlId });
}

export function enqueueDropped(dependencies: Dependencies) {
  const urlId = dependencies.urlId;
  return prepareDropped({ ...dependencies, urlId });
}

export function startAny(options: any) {
  return options;
}

export function structural(callback: () => Promise<unknown>) {
  callback();
}

export function takeString(options: { note: string }) {
  return options.note;
}

export function declaredSlot(dependencies: DeclaredDependencies): DeclaredDependencies { return dependencies; }
export function ambientSlot(dependencies: AmbientDependencies): AmbientDependencies { return dependencies; }
export function synchronousSlot(dependencies: SynchronousDependencies) { return dependencies; }
export function shadowedSlot(dependencies: ShadowedDependencies) { return dependencies; }
export function aliasedSlot(dependencies: AliasedDependencies): AliasedDependencies { return dependencies; }
export function partialSlot(dependencies: Partial<Dependencies>): Partial<Dependencies> { return dependencies; }
export function opaqueTyped(dependencies: Dependencies) { opaque(dependencies); }
export function opaqueMember(dependencies: Dependencies) { opaque(dependencies.x); }
export function untypedForward(dependencies: Dependencies) { untypedFinal({ ...dependencies }); }
declare function opaque(value: unknown): unknown;

export async function sibling(dependencies: { x: typeof enqueueEmail; other?: () => void }) {
  dependencies.other?.();
  await dependencies.x([], undefined);
}
export function evalSlot(dependencies: Dependencies) { eval('opaque(dependencies)'); return dependencies; }
export function argumentsSlot(dependencies: Dependencies) { return arguments[0]; }
export function identitySlot(dependencies: Dependencies) { return dependencies; }
export function typedIdentitySlot(dependencies: Dependencies): Dependencies { return dependencies; }
