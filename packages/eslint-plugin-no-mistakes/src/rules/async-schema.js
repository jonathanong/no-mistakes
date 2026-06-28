"use strict";

const targetOptionsSchema = [
  {
    type: "object",
    properties: {
      targets: {
        type: "array",
        items: {
          type: "object",
          properties: {
            sourcePatterns: { type: "array", items: { type: "string" } },
            calleeNamePatterns: { type: "array", items: { type: "string" } },
          },
          additionalProperties: false,
        },
      },
    },
    additionalProperties: false,
  },
];

module.exports = { targetOptionsSchema };
