import { it } from "vitest";
import { scan } from "./scan.mjs";

it("uses a computed fallback when the environment root is absent", () => scan());
