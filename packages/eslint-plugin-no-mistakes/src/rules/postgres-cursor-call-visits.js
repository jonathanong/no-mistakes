"use strict";

const { reportUnanalyzable, unanalyzablePolicy } = require("./postgres-unanalyzable");
const {
  namedCursorImport,
  namespaceCursorImport,
  namespaceCursorMember,
  namespaceImportMember,
} = require("./postgres-cursor-imports");
const { directCallParent, isTypeQuery, transparentParent } = require("./postgres-cursor-query");

function reportFinding(context, suppressed, descriptor) {
  if (suppressed(descriptor.node)) return;
  context.report(descriptor);
}

function reportDynamic(context, suppressed, node) {
  if (suppressed(node)) return;
  reportUnanalyzable(context, node, unanalyzablePolicy(context.options?.[0]));
}

function isTypeOnlyExport(identifier) {
  const specifier = identifier.parent;
  return (
    specifier?.type === "ExportSpecifier" &&
    (specifier.exportKind === "type" || specifier.parent?.exportKind === "type")
  );
}

function visitCursorIdentifier(context, node, options, reportedDirectUses, suppressed, helpers) {
  if (isTypeQuery(node) || isTypeOnlyExport(node)) return;
  const namedImport = namedCursorImport(context, node, helpers, options);
  const namespaceImport = namespaceCursorImport(context, node, helpers, options);
  if (namedImport === "unresolved") {
    const variable = helpers.findVariable(context, node);
    const reference = variable?.references.find((item) => item.identifier === node);
    if (reference && !reference.init && !directCallParent(node)) {
      reportDynamic(context, suppressed, node);
    }
    return;
  }
  if (namedImport == null && namespaceImport !== true) return;
  if (
    node.parent?.type === "ImportSpecifier" ||
    node.parent?.type === "ImportNamespaceSpecifier" ||
    (node.parent?.type === "TSImportEqualsDeclaration" && node.parent.id === node)
  ) {
    return;
  }
  const variable = helpers.findVariable(context, node);
  const reference = variable?.references.find((item) => item.identifier === node);
  if (!reference || reference.init) return;
  const expression = transparentParent(node);
  if (namespaceImport && namespaceImportMember(context, expression.parent, helpers, options)) {
    return;
  }
  if (!directCallParent(node) && !reportedDirectUses.has(node)) {
    reportedDirectUses.add(node);
    reportFinding(context, suppressed, { messageId: "directUse", node });
  }
}

function visitNamespaceMember(context, node, options, suppressed, helpers) {
  if (isTypeQuery(node)) return;
  const cursorMember = namespaceCursorMember(context, node, helpers, options);
  if (cursorMember === "unresolved") {
    if (!directCallParent(node)) reportDynamic(context, suppressed, node);
    return;
  }
  const namespaceMember = namespaceImportMember(context, node, helpers, options);
  if (!namespaceMember) return;
  if (namespaceMember.computed && helpers.propertyName(namespaceMember) == null) {
    reportFinding(context, suppressed, { messageId: "staticNamespaceMember", node });
    return;
  }
  if (cursorMember && !directCallParent(node)) {
    reportFinding(context, suppressed, { messageId: "directUse", node });
  }
}

module.exports = {
  reportDynamic,
  reportFinding,
  visitCursorIdentifier,
  visitNamespaceMember,
};
