// vi.fn() is a test double, not module mocking, so `.mock.test` is unwarranted (#388).
import { vi } from "vitest";
const onChange = vi.fn();
test("no module mocking", () => {
  onChange;
});
