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

const dispositionOptionsSchema = optionsSchema("targets");
dispositionOptionsSchema[0].properties.awaitingConsumers = {
  type: "array",
  items: {
    ...targetOptionsSchema[0].properties.targets.items,
    properties: {
      ...targetOptionsSchema[0].properties.targets.items.properties,
      callbackArgumentIndexes: {
        type: "array",
        minItems: 1,
        uniqueItems: true,
        items: { type: "integer", minimum: 0 },
      },
    },
    required: ["sourceSpecifierPatterns", "calleeNamePatterns", "callbackArgumentIndexes"],
  },
};

module.exports = { handlerOptionsSchema, targetOptionsSchema, dispositionOptionsSchema };
