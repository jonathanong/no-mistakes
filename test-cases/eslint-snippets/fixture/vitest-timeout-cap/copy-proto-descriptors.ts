const method = { __proto__() {} };
method.__proto__ = true;

const assertedSource = ({ __proto__() {} } as const);
const assertedAlias = assertedSource as typeof assertedSource;
assertedAlias.__proto__ = true;

const prototypeSetter = { __proto__: {} };
prototypeSetter.__proto__ = false;

const getter = { get __proto__() { return true; } };
getter.__proto__ = false;

const setter = { set __proto__(value: boolean) {} };
setter.__proto__ = false;

const computed = { ["__proto__"]: {} };
computed.__proto__ = true;

const __proto__ = {};
const shorthand = { __proto__ };
shorthand.__proto__ = true;
