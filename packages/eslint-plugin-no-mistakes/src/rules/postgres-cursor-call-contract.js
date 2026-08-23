"use strict";

const { rule } = require("../helpers");
const { findVariable, propertyName, unwrap } = require("./postgres-cursor-ast");
const { matchesCursorFile, resolveCursorContractOptions } = require("./postgres-cursor-options");
const {
  isCursorExecutor,
  isCursorModule,
  namedCursorImport,
  namespaceCursorImport,
  namespaceCursorMember,
  namespaceImportMember,
} = require("./postgres-cursor-imports");
const {
  directCallParent,
  isTypeQuery,
  queryHead,
  transparentParent,
} = require("./postgres-cursor-query");

const helpers = { findVariable, propertyName, unwrap };

function isTypeOnlyExport(identifier) {
  const specifier = identifier.parent;
  return (
    specifier?.type === "ExportSpecifier" &&
    (specifier.exportKind === "type" || specifier.parent?.exportKind === "type")
  );
}

function reExportsCursor(node, config) {
  if (!isCursorModule(node.source?.value, config) || node.exportKind === "type") return false;
  if (node.type === "ExportAllDeclaration") return true;
  return (
    node.specifiers?.some((specifier) => {
      const name = specifier.local?.name ?? specifier.local?.value;
      return specifier.exportKind !== "type" && isCursorExecutor(name, config);
    }) === true
  );
}

function visitCursorIdentifier(context, node, options, reportedDirectUses) {
  const namedImport = namedCursorImport(context, node, helpers, options);
  const namespaceImport = namespaceCursorImport(context, node, helpers, options);
  if (!namedImport && !namespaceImport) return;
  if (
    node.parent?.type === "ImportSpecifier" ||
    node.parent?.type === "ImportNamespaceSpecifier" ||
    isTypeQuery(node) ||
    isTypeOnlyExport(node)
  ) {
    return;
  }
  const variable = helpers.findVariable(context, node);
  if (!variable?.references.some((reference) => reference.identifier === node)) return;
  const expression = transparentParent(node);
  if (namespaceImport && namespaceImportMember(context, expression.parent, helpers, options)) {
    return;
  }
  if (!directCallParent(node) && !reportedDirectUses.has(node)) {
    reportedDirectUses.add(node);
    context.report({ messageId: "directUse", node });
  }
}

function visitNamespaceMember(context, node, options) {
  const namespaceMember = namespaceImportMember(context, node, helpers, options);
  if (!namespaceMember || isTypeQuery(node)) return;
  if (namespaceMember.computed && helpers.propertyName(namespaceMember) == null) {
    context.report({ messageId: "staticNamespaceMember", node });
    return;
  }
  if (namespaceCursorMember(context, node, helpers, options) && !directCallParent(node)) {
    context.report({ messageId: "directUse", node });
  }
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
    },
  },
  (context) => {
    const options = resolveCursorContractOptions(context.options[0]);
    if (!options || !matchesCursorFile(context, options)) return {};
    const reportedDirectUses = new WeakSet();
    return {
      CallExpression(node) {
        const callee = helpers.unwrap(node.callee);
        const executor =
          namedCursorImport(context, callee, helpers, options) ||
          namespaceCursorMember(context, callee, helpers, options);
        if (!executor) return;
        const argument = node.arguments?.[0];
        const reportNode = argument ?? node;
        const head = argument ? queryHead(context, argument, helpers, options) : null;
        if (head === null) {
          context.report({ messageId: "staticQuery", node: reportNode });
          return;
        }
        if (!options.annotation.test(head)) {
          context.report({ messageId: "annotation", node: reportNode });
        }
      },
      Identifier(node) {
        visitCursorIdentifier(context, node, options, reportedDirectUses);
      },
      MemberExpression(node) {
        visitNamespaceMember(context, node, options);
      },
      ExportAllDeclaration(node) {
        if (reExportsCursor(node, options)) context.report({ messageId: "directUse", node });
      },
      ExportNamedDeclaration(node) {
        if (reExportsCursor(node, options)) context.report({ messageId: "directUse", node });
      },
    };
  },
);
