import { configDefaults, defineConfig } from "vitest/config";

declare function getExtra(): string[];

export default defineConfig({ test: { exclude: [...configDefaults.exclude, ...getExtra()] } });
