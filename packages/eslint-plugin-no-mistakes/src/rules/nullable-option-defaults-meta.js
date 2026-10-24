"use strict";

module.exports = {
  type: "problem",
  docs: {
    description: "preserve explicit null in nullable option defaults",
    recommended: false,
  },
  schema: [
    {
      type: "object",
      properties: {
        includePathPatterns: { type: "array", items: { type: "string" } },
        excludePathPatterns: { type: "array", items: { type: "string" } },
        optionObjectNames: { type: "array", items: { type: "string" } },
        optionObjectNamePatterns: { type: "array", items: { type: "string" } },
      },
      additionalProperties: false,
    },
  ],
  messages: {
    default:
      "Do not default nullable option '{{name}}' with ??, ||, ??=, or ||=. Preserve explicit null and check undefined explicitly.",
  },
};
