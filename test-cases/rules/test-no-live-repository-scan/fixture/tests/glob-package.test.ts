// Positive: glob packages enumerate relative to the working directory.
import fg from "fast-glob";
import { globSync } from "glob";

globSync(".github/workflows/*.yml");
fg.sync("src/**/*.ts", { cwd: process.cwd() });
