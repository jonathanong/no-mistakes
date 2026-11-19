"use strict";

const patterns = [
  "referenceNamePattern",
  "declarationNamePattern",
  "propertyNamePattern",
  "destructureNamePattern",
];
const strictRequired = [...patterns, "constructNamePattern"].map((key) => ({ required: [key] }));

const targetSchema = {
  type: "object",
  properties: {
    source: { type: "string", minLength: 1 },
    importedNamePattern: { type: "string", minLength: 1 },
    callNamePattern: { type: "string", minLength: 1 },
    discardedResultOnly: { type: "boolean" },
    constructNamePattern: { type: "string", minLength: 1 },
    constructArgumentCount: { type: "integer", minimum: 0 },
    ...Object.fromEntries(patterns.map((key) => [key, { type: "string", minLength: 1 }])),
    message: { type: "string", minLength: 1 },
  },
  anyOf: [
    { required: ["importedNamePattern"] },
    { required: ["callNamePattern"] },
    ...strictRequired,
  ],
  dependencies: {
    source: ["importedNamePattern"],
    constructArgumentCount: ["constructNamePattern"],
    discardedResultOnly: ["callNamePattern"],
  },
  additionalProperties: false,
};

module.exports = { targetSchema };
