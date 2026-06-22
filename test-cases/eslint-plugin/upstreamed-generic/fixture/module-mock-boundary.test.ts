import { mock } from "vitest";
import { vi } from "vitest";

const mockModule = vi.mock;

vi.mock("@app/service", () => ({ run: vi.fn() }));
mock("@app/aliased", () => ({ run: vi.fn() }));
mockModule.call(undefined, "@app/call", () => ({ run: vi.fn() }));
mockModule.apply(undefined, ["@app/apply", () => ({ run: vi.fn() })]);
vi.mock(`@app/${name}`, () => ({ run: vi.fn() }));

vi.mock("@app/baselined", () => ({ run: vi.fn() }));

vi.mock("@app/integration", () => ({ allowed: vi.fn() }));
vi.mock("external-package", () => ({ run: vi.fn() }));
