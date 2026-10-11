"use strict";
module.exports = {
  type: "problem",
  docs: { description: "require bounded positive Playwright test deadlines", recommended: false },
  schema: [
    {
      type: "object",
      properties: {
        max: { type: "number", minimum: 0, exclusiveMinimum: true },
        fixtureMax: { type: "number", minimum: 0, exclusiveMinimum: true },
        unknownValues: { enum: ["ignore", "finding"] },
        registrationPackages: { type: "array", items: { type: "string" }, uniqueItems: true },
        exportRoles: {
          type: "object",
          additionalProperties: {
            type: "object",
            additionalProperties: { enum: ["assertion", "ordinary", "opaque"] },
          },
        },
        configFiles: { type: "array", items: { type: "string" }, uniqueItems: true },
      },
      additionalProperties: false,
    },
  ],
  messages: {
    effectiveSlot:
      "Playwright registration inherits an unresolved effective deadline/latch; prove its configured owner and helper slots against {{max}} ms.",
    invalid: "Playwright {{name}} must be positive and finite; zero disables the deadline.",
    timeout:
      "Playwright {{name}} exceeds {{max}} ms. Fix the stalled work rather than increasing the deadline.",
    unknown:
      "Playwright {{name}} cannot be resolved against {{max}} ms; keep admitted timeout carriers explicit.",
  },
};
