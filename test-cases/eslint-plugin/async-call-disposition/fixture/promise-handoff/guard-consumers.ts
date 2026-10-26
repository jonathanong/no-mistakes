import { enqueueEmail } from '@app/jobs';
import type * as G from './guard-types';
import type { Dependencies, Job, Structural, NoX, StableDependencies, MutableDependencies, ArrowDependencies, GeneratorDependencies, SyncDependencies, ShadowDependencies, DeepDependencies, DeepAliasDependencies } from './guard-types';
export function stable(d: StableDependencies): StableDependencies { return d; }
export function mutable(d: MutableDependencies): MutableDependencies { return d; }
export function arrow(d: ArrowDependencies): ArrowDependencies { return d; }
export function generator(d: GeneratorDependencies): GeneratorDependencies { return d; }
export function synchronous(d: SyncDependencies): SyncDependencies { return d; }
export function shadow(d: ShadowDependencies): ShadowDependencies { return d; }
export function deep(d: DeepDependencies): DeepDependencies { return d; }
export function deepAlias(d: DeepAliasDependencies): DeepAliasDependencies { return d; }
export function identity(d: Dependencies): Dependencies { return d; }
export function namespaceType(d: G.Dependencies): G.Dependencies { return d; }
export function typedAlias(d: Dependencies): Dependencies { const alias: Dependencies = d; return alias; }
export function callbackAlias(d: Dependencies): Job { const alias: Job = d.x; return alias; }
export function anyAlias(d: Dependencies): Dependencies { const alias: any = d; return alias; }
export function wrongAlias(d: Dependencies): Structural { const alias: Structural = d; return alias; }
export function missingAlias(d: Dependencies): NoX { const alias: NoX = d; return alias; }
export function wrongCallbackAlias(d: Dependencies): () => Promise<void> { const alias: () => Promise<void> = d.x; return alias; }
export function assertion(d: Dependencies): Job { return d.x as Job; }
export function wrongAssertion(d: Dependencies): Job { return d.x as () => Promise<void>; }
export const arrowIdentity = (d: Dependencies): Dependencies => d;
export const unknownReturn = (d: Dependencies): unknown => d;
export function directCallback(x: Job): Job { return x; }
export function structuralCallback(x: Job): () => Promise<void> { return x; }
export function dynamic(d: Dependencies, key: 'x' = 'x') { d[key]([], undefined); }
export function protectedMember(d: Dependencies) { void d.constructor; void d.x([], undefined); }
export function optionalReceiver(d: Dependencies) { void d?.x([], undefined); }
export function optionalCall(d: Dependencies) { void d.x?.([], undefined); }
export function chainedSibling(d: Dependencies) { void d.note?.length; void d.x([], undefined); }
export function unused(d: Dependencies) { void 0; }
export function branchAlias(d: Dependencies, condition = true) { const alias = condition ? d.x : enqueueEmail; return alias; }
export function mutableAlias(d: Dependencies) { let alias = d; return alias; }
export function arrayCarrier(d: Dependencies) { return [d.x]; }
export function duplicate(d: Dependencies) { return identity({ x: d.x, x: enqueueEmail }); }
export function getter(d: Dependencies) { return identity({ x: d.x, get note() { return 'getter'; } }); }
export function method(d: Dependencies) { return identity({ x: d.x, other() {} }); }
export function proto(d: Dependencies) { return identity({ x: d.x, __proto__: null }); }
export function computed(d: Dependencies) { return identity({ ['x']: d.x }); }
export function override(d: Dependencies) { return identity({ ...d, x: enqueueEmail }); }
export function spreadProto(d: Dependencies) { return identity({ ...d, __proto__: null }); }
export function otherSpread(d: Dependencies) { return identity({ ...d, ...{} }); }
export function arraySpread(d: Dependencies) { return [...d]; }
export function correctFresh(d: Dependencies): Dependencies { return { x: d.x, note: d.note }; }
export function correctSpread(d: Dependencies): Dependencies { return { ...d, note: 'copied' }; }
export function fallback(d: Dependencies): Job { return d.x ?? enqueueEmail; }
export function unknownFallback(d: Dependencies): Job { return d.x ?? (async () => {}); }
export function returnedArrow(d: Dependencies) { return (): Job => d.x; }
export function unsupportedExpression(d: Dependencies) { void d.x; }
