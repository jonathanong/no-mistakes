// Test-only unknown constructions never count as production uncertainty.
export function build(ctor: any) { return new ctor(); }
