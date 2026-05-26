import { vi } from "vitest";
const myFn = vi.fn();
myFn.mockReturnValue(42);
test("uses mock chain method", () => {});
