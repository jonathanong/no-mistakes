function mixin(base: any): any { return base; }
class Base {}
// An Error argument does not prove the returned constructor preserves Error.
export class MixedError extends mixin(Error) {}
export class MixedBase extends mixin(Base) {}
export class ExpressionBase extends (true ? Error : Base) {}
export class ComputedBase extends ({ Error })[String("Error")] {}
export class MixedParent extends mixin(Error) {}
export class StaticChild extends MixedParent {}
export class Constructed extends mixin(Error) {}
new Constructed();
export class Subclassed extends mixin(Error) {}
class Child extends Subclassed {}
export declare class Ambient extends mixin(Error) {}
declare namespace External { export class AmbientNested extends mixin(Error) {} }
export class DefinitelyDead extends Error {}
// no-mistakes-disable-next-line unconstructed-error-class: intentional dynamic base
export class Suppressed extends mixin(Error) {}
export namespace Errors { export class Dynamic extends mixin(Error) {} }
// Dynamic class expressions remain outside declaration ancestry analysis.
export const Expression = class extends mixin(Error) {};
