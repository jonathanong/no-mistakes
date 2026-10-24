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
      callbackArgumentProperties: {
        type: "array",
        minItems: 1,
        items: {
          type: "object",
          properties: {
            argumentIndex: { type: "integer", minimum: 0 },
            propertyNames: {
              type: "array",
              minItems: 1,
              uniqueItems: true,
              items: { type: "string", minLength: 1 },
            },
          },
          required: ["argumentIndex", "propertyNames"],
          additionalProperties: false,
        },
      },
    },
    required: ["sourceSpecifierPatterns", "calleeNamePatterns"],
    anyOf: [
      { required: ["callbackArgumentIndexes"] },
      { required: ["callbackArgumentProperties"] },
    ],
  },
};

module.exports = { handlerOptionsSchema, targetOptionsSchema, dispositionOptionsSchema };
