// Negative: no Vitest root reaches this file, so a live scan here is out of scope.
import { readdirSync } from "node:fs";

export const names = readdirSync(".github/workflows");
