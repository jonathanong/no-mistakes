import { defineConfig } from "vitest/config";

const changed = ["**/.git/**"];
changed.pop();

export default defineConfig({
  test: {
    exclude: ["**/node_modules/**", ...["**/.git/**"]],
    projects: [{ test: { exclude: ["**/node_modules/**", ...changed] } }],
  },
});

const patterns = ["**/.git/**"];
export function createConfig(patterns: string[]) {
  return { test: { exclude: ["**/node_modules/**", ...patterns] } };
}
export const complete = { test: { exclude: ["**/node_modules/**", ...patterns] } };
