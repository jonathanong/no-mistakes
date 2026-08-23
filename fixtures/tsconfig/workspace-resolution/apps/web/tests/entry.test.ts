import { expect, it } from "vitest";
import { webEntry } from "../src/entry";

it("uses the web entry", () => expect(webEntry).toBe("web:shared"));
