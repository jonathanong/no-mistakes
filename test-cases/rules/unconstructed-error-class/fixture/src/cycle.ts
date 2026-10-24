// Each cyclic class is subclassed, so neither is reported as unconstructed.
// Ancestry remains uncertain; the fixed-point walk must still terminate.
export class CycleA extends CycleB {}
export class CycleB extends CycleA {}
