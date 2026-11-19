import { describe, expect, it } from "vitest";
import { fixture, lint } from "./helpers.mjs";

const id = "require-options-on-imported-call";
const base = { optionsPosition: 2, requiredProperties: ["agent"] };
const files = {
  "value-validation": ["@example/transport", "send"],
  "value-validation-default": ["@example/transport", "send"],
  "value-validation-namespace": ["@example/transport", "send"],
  "value-validation-renamed": ["@example/transport", "send"],
};
const modes = {
  off: {},
  on: { allowedValueCallees: { agent: ["approvedAgent"] }, maxArguments: 2 },
  "on+shorthand": {
    allowedValueCallees: { agent: ["approvedAgent"] },
    allowShorthand: true,
    maxArguments: 2,
  },
};

function run(name, extra) {
  const [source, callee] = files[name];
  const target = {
    sourceSpecifierPatterns: [source],
    calleeNamePatterns: [callee],
    ...base,
    ...extra,
  };
  return lint(
    fixture(`${id}/${name}.mjs`),
    { [`no-mistakes/${id}`]: ["error", { targets: [target] }] },
    `${name}.mjs`,
  ).map((m) => `${m.line}:${m.messageId}`);
}

describe("require-options-on-imported-call value validation", () => {
  it("snapshots every import shape with the options off, on, and shorthand allowed", async () => {
    const sections = [];
    for (const name of Object.keys(files)) {
      for (const [label, extra] of Object.entries(modes)) {
        sections.push(`## ${name} (${label})\n${run(name, extra).join("\n")}\n`);
      }
    }
    await expect(sections.join("\n")).toMatchFileSnapshot(
      "snapshots/require-options-value-validation.snap",
    );
  });

  it("keeps today's output when the new options are absent", () => {
    for (const name of Object.keys(files)) expect(run(name, modes.off)).toEqual([]);
  });
});
