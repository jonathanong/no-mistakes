import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { messages } from "./helpers.mjs";

const rule = "no-three-sequential-awaits";

describe("no-three-sequential-awaits", () => {
  it("reports three sequential awaits", () => {
    const code = `
      async function run() {
        await one();
        await two();
        await three();
      }
    `;
    assert.deepEqual(messages(code, rule, undefined, "example.ts"), ["sequential"]);
  });

  it("reports mixed declaration and assignment awaits", () => {
    const code = `
      async function run() {
        const one = await loadOne();
        const two = await loadTwo();
        target.value = await loadThree();
      }
    `;
    assert.deepEqual(messages(code, rule, undefined, "example.ts"), ["sequential"]);
  });

  it("allows two sequential awaits", () => {
    const code = `
      async function run() {
        await one();
        await two();
      }
    `;
    assert.deepEqual(messages(code, rule, undefined, "example.ts"), []);
  });

  it("resets the run across a non-await statement", () => {
    const code = `
      async function run() {
        await one();
        if (condition) await two();
        await three();
      }
    `;
    assert.deepEqual(messages(code, rule, undefined, "example.ts"), []);
  });

  it("ignores Promise.all and Promise.allSettled", () => {
    const code = `
      async function run() {
        await Promise.all([one(), two()]);
        await Promise.allSettled([three()]);
        await four();
      }
    `;
    assert.deepEqual(messages(code, rule, undefined, "example.ts"), []);
  });

  it("reports each overlapping window of three awaits", () => {
    const code = `
      async function run() {
        await one();
        await two();
        await three();
        await four();
      }
    `;
    assert.deepEqual(messages(code, rule, undefined, "example.ts"), ["sequential", "sequential"]);
  });

  it("scans program bodies, switch cases, and Promise.all assignments", () => {
    assert.deepEqual(
      messages(
        `
      await one();
      await two();
      await three();
    `,
        rule,
        undefined,
        "example.mts",
      ),
      ["sequential"],
    );
    assert.deepEqual(
      messages(
        `
      async function run(value) {
        switch (value) {
          case 1:
            await one();
            await two();
            await three();
        }
      }
    `,
        rule,
        undefined,
        "example.ts",
      ),
      ["sequential"],
    );
    assert.deepEqual(
      messages(
        `
      async function run() {
        const packed = await Promise.all([one(), two()]);
        target = await Promise.allSettled([three()]);
        await four();
        let skipped;
        var a = 1, b = await five();
        await Promise["all"]([six()]);
      }
    `,
        rule,
        undefined,
        "example.ts",
      ),
      [],
    assert.deepEqual(
      messages(
        `
      async function run() {
        const one = await (loadOne() as Promise<number>);
        let two = await loadTwo();
        var three = await loadThree();
      }
    `,
        rule,
        undefined,
        "example.ts",
      ),
      ["sequential"],
    );
  });
});
