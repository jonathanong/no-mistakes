// Not flagged: a cyclic `extends` never reaches the built-in Error, and the
// walk must still terminate.
export class CycleA extends CycleB {}
export class CycleB extends CycleA {}
