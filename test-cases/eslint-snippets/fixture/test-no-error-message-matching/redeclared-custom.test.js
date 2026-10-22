// Both declarations select ordinary functions; neither introduces runner expect.
var check = (value) => ({ toBe: (other) => value === other });
var check = (value) => ({ toBe: (other) => value === other });
const err = new Error("details");
check(err.message).toBe("details");
