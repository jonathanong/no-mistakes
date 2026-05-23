"use strict";

const {
  collectExportedComponents,
  normalizedComponentOptions,
  shouldCheckComponent,
} = require("../exported-components");
const { nonEmptyStrings } = require("../component-functions");
const { options, rule } = require("../helpers");
const { jsxTreeHasAttribute, returnedJsxBranches } = require("../returned-jsx");

const DEFAULT_ATTRIBUTES = ["data-pw"];

module.exports = rule(
  {
    type: "suggestion",
    docs: {
      description: "require configured attributes in exported component JSX trees",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        properties: {
          attributes: { type: "array", items: { type: "string" } },
          componentNamePattern: { type: "string" },
          components: { type: "array", items: { type: "string" } },
          ignoreComponents: { type: "array", items: { type: "string" } },
          wrappers: { type: "array", items: { type: "string" } },
          allowSpreadAttributes: { type: "boolean" },
          exportTypes: {
            type: "array",
            items: { enum: ["named", "default"] },
          },
          checkAnonymousDefault: { type: "boolean" },
        },
        additionalProperties: false,
      },
    ],
    messages: {
      missing: "Exported component '{{name}}' must return JSX containing one of: {{attributes}}.",
    },
  },
  (context) => ({
    Program(node) {
      const opts = normalizedOptions(options(context));
      for (const component of collectExportedComponents(node, opts)) {
        reportMissingBranches(context, component, opts);
      }
    },
  }),
);

function normalizedOptions(option) {
  return {
    ...normalizedComponentOptions(option),
    attributes: nonEmptyStrings(option.attributes, DEFAULT_ATTRIBUTES),
    allowSpreadAttributes: option.allowSpreadAttributes === true,
  };
}

function reportMissingBranches(context, component, opts) {
  if (shouldCheckComponent(component, opts)) {
    for (const jsx of returnedJsxBranches(component.fn)) {
      if (!jsxTreeHasAttribute(jsx, opts)) {
        context.report({
          node: jsx,
          messageId: "missing",
          data: {
            name: component.name,
            attributes: opts.attributes.join(", "),
          },
        });
      }
    }
  }
}
