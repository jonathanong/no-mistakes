"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { findVariable, propertyName, unwrap } = require("./postgres-cursor-ast");
const { matchesCursorFile, resolveCursorContractOptions } = require("./postgres-cursor-options");
const { resolveModuleBinding } = require("./postgres-module-binding");
const {
  isCursorExecutor,
  isCursorModule,
  namedCursorImport,
  namespaceCursorMember,
} = require("./postgres-cursor-imports");
const {
  reportDynamic,
  reportFinding,
  visitCursorIdentifier,
  visitNamespaceMember,
} = require("./postgres-cursor-call-visits");
const { UNRESOLVED_QUERY, queryHead } = require("./postgres-cursor-query");

const helpers = { findVariable, propertyName, unwrap };

function reExportsCursor(node, context, config) {
  if (node.exportKind === "type") return false;
  if (node.declaration?.type === "TSImportEqualsDeclaration") {
    const binding = resolveModuleBinding(node.declaration.id, context);
    return (
      binding?.status === "resolved" &&
      binding.kind === "namespace" &&
      isCursorModule(binding.source, config)
    );
  }
  if (!isCursorModule(node.source?.value, config)) return false;
  if (node.type === "ExportAllDeclaration") return true;
  return (
    node.specifiers?.some((specifier) => {
      const name = specifier.local?.name ?? specifier.local?.value;
      return specifier.exportKind !== "type" && isCursorExecutor(name, config);
    }) === true
  );
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "require direct cursor calls with statically annotated SQL",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        additionalProperties: false,
        properties: {
          modules: { type: "array", items: { type: "string" } },
          executors: { type: "array", items: { type: "string" } },
          include: { type: "array", items: { type: "string" } },
          exclude: { type: "array", items: { type: "string" } },
          includeFiles: { type: "array", items: { type: "string" } },
          annotation: { type: "string" },
          sqlTagModules: { type: "array", items: { type: "string" } },
          trustedSqlTags: {
            type: "array",
            items: {
              type: "object",
              additionalProperties: false,
              required: ["module", "name"],
              properties: {
                module: { type: "string" },
                name: { type: "string" },
              },
            },
          },
          unanalyzable: { enum: ["advise", "report", "ignore"] },
        },
      },
    ],
    messages: {
      annotation: "PostgreSQL cursor SQL must start with a static /* name */ annotation.",
      directUse: "PostgreSQL cursor helpers must be called directly so their SQL can be verified.",
      staticQuery:
        "PostgreSQL cursor SQL must be visible at the callsite or in one immutable local binding.",
      staticNamespaceMember:
        "PostgreSQL namespace members must use a static property name so cursor use can be verified.",
      unanalyzable:
        "This PostgreSQL cursor value is not a static import, require, or import-equals binding. Write the module specifier as a string literal so the SQL can be checked. A dynamic module is not a clean result.",
    },
  },
  (context) => {
    const options = resolveCursorContractOptions(context.options[0]);
    if (!options || !matchesCursorFile(context, options)) return {};
    const suppressed = ruleSuppression(context, "postgres-cursor-call-contract");
    const reportedDirectUses = new WeakSet();
    return {
      CallExpression(node) {
        const callee = helpers.unwrap(node.callee);
        const named = namedCursorImport(context, callee, helpers, options);
        const member = namespaceCursorMember(context, callee, helpers, options);
        if (named === "unresolved" || member === "unresolved") {
          reportDynamic(context, suppressed, node);
          return;
        }
        const executor = named || member;
        if (!executor) return;
        const argument = node.arguments?.[0];
        const reportNode = argument ?? node;
        const head = argument ? queryHead(context, argument, helpers, options) : null;
        if (head === UNRESOLVED_QUERY) {
          reportDynamic(context, suppressed, reportNode);
          return;
        }
        if (head === null) {
          reportFinding(context, suppressed, { messageId: "staticQuery", node: reportNode });
          return;
        }
        if (!options.annotation.test(head)) {
          reportFinding(context, suppressed, { messageId: "annotation", node: reportNode });
        }
      },
      Identifier(node) {
        visitCursorIdentifier(context, node, options, reportedDirectUses, suppressed, helpers);
      },
      MemberExpression(node) {
        visitNamespaceMember(context, node, options, suppressed, helpers);
      },
      ExportAllDeclaration(node) {
        if (reExportsCursor(node, context, options)) {
          reportFinding(context, suppressed, { messageId: "directUse", node });
        }
      },
      ExportNamedDeclaration(node) {
        if (reExportsCursor(node, context, options)) {
          reportFinding(context, suppressed, { messageId: "directUse", node });
        }
      },
    };
  },
);
