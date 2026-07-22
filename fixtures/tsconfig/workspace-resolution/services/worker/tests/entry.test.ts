import { expect, it } from "vitest";
import { workerEntry } from "../src/entry";

it("uses the worker entry", () => expect(workerEntry).toBe("worker:shared"));
