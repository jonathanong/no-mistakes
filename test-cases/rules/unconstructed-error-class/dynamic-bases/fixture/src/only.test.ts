function mixin(base: any): any { return base; }
export class TestClass extends mixin(Error) {}
