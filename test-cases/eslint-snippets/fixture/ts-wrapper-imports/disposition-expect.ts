import enqueue from "@example/jobs";
import * as vt from "vitest";
import eq = require("vitest");

const { expect } = require("vitest");

enqueue();
await expect(enqueue()).rejects.toThrow();
await vt.expect(enqueue()).rejects.toThrow();
await eq.expect(enqueue()).rejects.toThrow();
