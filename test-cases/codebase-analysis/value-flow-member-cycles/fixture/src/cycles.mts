// Invalid initializers remain valid static-analysis inputs: never recurse forever.
const direct = { method: direct.method };
direct.method();

const left = { method: right.method };
const right = { method: left.method };
left.method();
right.method();

const slots = [slots[0]];
slots[0]();
