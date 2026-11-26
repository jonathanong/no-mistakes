import { vi } from "vitest";
const factory = () => ({ widget: () => 1 });

vi.mock("./widgets", factory);
(vi.mock as typeof vi.mock)("./widgets", factory);
vi.mock!("./widgets", factory);
(vi.mock satisfies typeof vi.mock)("./widgets", factory);
(vi.mock as typeof vi.mock).call(vi, "./widgets", factory);
((0, vi.mock)! as typeof vi.mock)("./widgets", factory);
(vi.mock satisfies typeof vi.mock)("./widgets", async (importOriginal) => ({
  ...(await importOriginal<object>()),
  widget: () => 1,
}));
(vi.mock as typeof vi.mock)("vitest", factory);
