import { expect, test } from "vitest";
import { label } from "../src/label";

test("default label", () => expect(label).toBe("synthetic"));
