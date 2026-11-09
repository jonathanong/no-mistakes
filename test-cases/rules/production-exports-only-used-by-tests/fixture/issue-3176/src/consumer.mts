import { readdirSync } from "node:fs";
import { join } from "node:path";
import { createRequire } from "node:module";
import { createRequire as makeRequire } from "node:module";
import { pathToFileURL } from "node:url";

const toolsDirectory = new URL("./tools/", import.meta.url).pathname;
for (const file of readdirSync(toolsDirectory).filter((entry) => entry.endsWith(".mts"))) {
  await import(pathToFileURL(join(toolsDirectory, file)).href);
}

const require = createRequire(import.meta.url);
await import(pathToFileURL(require.resolve("some-package/runtime-entry")).href);
await import(pathToFileURL(require.resolve("another-package")).href);
await import(pathToFileURL(makeRequire(require.resolve("vitest/node")).resolve("vite")).href);
