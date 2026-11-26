import { defineConfig } from "vitest/config";

const extra = ["**/*.slow.test.ts"];
export default defineConfig({ test: { exclude: ["**/node_modules/**", ...extra] } });
