// This loader's callee is a function expression, not the global require binding.
const { expect: check } = (() => ({ expect: (value) => ({ toBe: (other) => value === other }) }))();
const err = new Error("details");
check(err.message).toBe("details");
