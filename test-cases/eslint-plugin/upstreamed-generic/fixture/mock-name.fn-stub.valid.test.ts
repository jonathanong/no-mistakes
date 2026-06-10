// Regression (issue #388): typed vi.fn<T>() callback stubs and `.mock*()` chain
// helpers are test doubles, not module mocks, so a plain `.test.ts` is correct.
import { vi } from "vitest";
const onChange = vi.fn<(value: string) => void>();
onChange.mockReturnValue(undefined);
test("typed callback stub needs no .mock.test filename", () => {});
