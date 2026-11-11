import { readdirSync } from "node:fs";
import { vi } from "vitest";

const captured = process.env.ROOT ?? "./tmp/example";
readdirSync(captured);
readdirSync(process.env.ROOT || "./tmp/or");
vi.stubEnv("ROOT", undefined);
delete process.env.ROOT;
const computed = process.env.ROOT ?? makeFallback();
readdirSync(computed);

function makeFallback() {
  return "./tmp/fallback";
}
