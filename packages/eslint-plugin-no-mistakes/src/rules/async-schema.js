"use strict";

function optionsSchema(propertyName) {
  return [
    {
      type: "object",
      properties: {
        [propertyName]: {
          type: "array",
          items: {
            type: "object",
            properties: {
              sourceSpecifierPatterns: { type: "array", items: { type: "string" } },
              calleeNamePatterns: { type: "array", items: { type: "string" } },
            },
            additionalProperties: false,
          },
        },
      },
      additionalProperties: false,
    },
  ];
}

const handlerOptionsSchema = optionsSchema("handlers");
const targetOptionsSchema = optionsSchema("targets");

module.exports = { handlerOptionsSchema, targetOptionsSchema };
