const { unused, ["expect"]: check = fallback } = require("vitest");
const err = new Error("details");
check(err.message).toBe("details");
