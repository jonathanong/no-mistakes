import { it } from "vitest";
import { scan } from "./scan.mjs";

it("scans without assigning the environment root", () => scan());
