"use strict";

const patterns = ["referenceNamePattern", "declarationNamePattern", "propertyNamePattern"];
const strictRequired = [...patterns, "constructNamePattern"].map((key) => ({ required: [key] }));

const targetSchema = {
  type: "object",
  properties: {
    source: { type: "string", minLength: 1 },
    importedNamePattern: { type: "string", minLength: 1 },
    callNamePattern: { type: "string", minLength: 1 },
    constructNamePattern: { type: "string", minLength: 1 },
    constructArgumentCount: { type: "integer", minimum: 0 },
    ...Object.fromEntries(patterns.map((key) => [key, { type: "string", minLength: 1 }])),
    message: { type: "string", minLength: 1 },
  },
  anyOf: [
    { required: ["source", "importedNamePattern"] },
    { required: ["callNamePattern"] },
    ...strictRequired,
  ],
  dependencies: {
    importedNamePattern: ["source"],
    source: ["importedNamePattern"],
    constructArgumentCount: ["constructNamePattern"],
  },
  additionalProperties: false,
};

module.exports = { targetSchema };
