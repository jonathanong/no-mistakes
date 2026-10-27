import { vi } from "vitest";
// Literal mocked modules have statically known targets, even without production consumers.
vi.mock(import("../src/mock-only.mjs"), () => ({ mockOnlyValue: () => 2, second: function () { return 3; }, settings: { enabled: true, label: "mock", absent: null, count: 1, large: 1n }, values: [1, "two", false, null, { nested: [] }] }));
// Both supported mock methods accept a static function-expression replacement.
vi.doMock(import("../src/mock-only.mjs"), function () { return { mockOnlyValue: () => 3 }; });
