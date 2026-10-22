import { "expect" as check } from "vitest";
const err = new Error("details");
check(err.message).toBe("details");
