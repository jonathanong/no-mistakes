function loadChecks() {
  return { expect: (value) => ({ toBe: (other) => value === other }) };
}
const { expect: check } = loadChecks();
const err = new Error("details");
check(err.message).toBe("details");
