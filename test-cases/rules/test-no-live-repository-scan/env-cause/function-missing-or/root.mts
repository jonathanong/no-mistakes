import { it } from "vitest";
import { scan } from "./scan.mjs";

it("uses the literal fallback when the environment root is absent", () => scan());
