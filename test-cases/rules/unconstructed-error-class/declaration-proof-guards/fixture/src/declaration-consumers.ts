import facade, { Constructor, ErrorConstructor, Whole, Bare } from 'guarded-constructors';
export class Known extends Constructor {}
export class Required extends facade.Required {}
export class Nested extends facade.Nested {}
export class DefiniteUnusedError extends ErrorConstructor {}
export class PropertyUnusedError extends facade.ErrorRequired {}
export class OptionalBase extends facade.Optional {}
export class ComputedBase extends facade.Computed {}
export class SignatureBase extends facade.Bare {}
export class MethodBase extends facade.Method {}
export class UntypedBase extends facade.Untyped {}
export class TypedBase extends facade.Typed {}
export class ModuleQueryBase extends Whole.Deep.Root {}
export class BareSignatureBase extends Bare {}
