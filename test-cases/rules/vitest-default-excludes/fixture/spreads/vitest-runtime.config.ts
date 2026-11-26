import { defineConfig } from "vitest/config";

declare function loadPatterns(): string[];
const runtimePatterns = loadPatterns();
export default defineConfig({ test: { exclude: ["**/node_modules/**", ...runtimePatterns] } });
