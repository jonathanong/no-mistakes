import { enqueueEmail } from '@app/jobs';
import type { Dependencies } from './types';
import type { LiteralDependencies, MissingDependencies, OverloadedDependencies, ExtendedDependencies, QuotedDependencies, MergedDependencies } from './extra-types';
export function defaultSlot(d: Dependencies = { x: enqueueEmail }): Dependencies { return d; }
export function literalSlot(d: LiteralDependencies): LiteralDependencies { return d; }
export function missingSlot(d: MissingDependencies): MissingDependencies { return d; }
export function overloadedSlot(d: OverloadedDependencies): OverloadedDependencies { return d; }
export function extendedSlot(d: ExtendedDependencies): ExtendedDependencies { return d; }
export function quotedSlot(d: QuotedDependencies): QuotedDependencies { return d; }
export function mergedSlot(d: MergedDependencies): MergedDependencies { return d; }
export let mutableConsumer = (d: Dependencies): Dependencies => d;
mutableConsumer = (d) => d;
