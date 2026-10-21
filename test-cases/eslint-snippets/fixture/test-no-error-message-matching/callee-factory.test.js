function makeCheck() {
  return (value) => ({ toBe: (other) => value === other });
}
const err = new Error("details");
makeCheck()(err.message).toBe("details");
