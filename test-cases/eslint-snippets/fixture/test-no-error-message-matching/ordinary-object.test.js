// An ordinary object's method is not a test-runner assertion binding.
const { expect: check } = { expect: (value) => ({ toBe: (other) => value === other }) };
const err = new Error("details");
check(err.message).toBe("details");
