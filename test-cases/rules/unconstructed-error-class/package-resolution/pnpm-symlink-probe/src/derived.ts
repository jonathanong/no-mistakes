import { Class as BareClass } from "pkg-bare";
import { Class as RelativeClass } from "pkg-relative";
export class DerivedBare extends BareClass {}
export class DerivedRelative extends RelativeClass {}
import { Class as WithinClass } from "pkg-within";
export class SafeWithin extends WithinClass {}
// no-mistakes-disable-next-line unconstructed-error-class -- Intentional opaque package boundary.
export class SuppressedBare extends BareClass {}
