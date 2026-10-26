import type { EnqueueReturnType as Ambiguous } from './ambiguous-index';
import type { EnqueueReturnType as Unknown } from './unknown-index';
type Union = Promise<unknown> | undefined;
type Generic<T> = Promise<T>;
import type { Shadowed } from './shadowed-return';
export function enqueueAmbiguous(): Ambiguous { throw new Error(); }
export function enqueueUnknown(): Unknown { throw new Error(); }
export function enqueueUnion(): Union { throw new Error(); }
export function enqueueGeneric(): Generic<string> { throw new Error(); }
export function enqueueShadowed(): Shadowed { throw new Error(); }
