"use strict";

const { resolveVariable } = require("./postgres-query-text");

function configuredTag(source, name, options) {
  return options.trustedSqlTags.some(
    (tag) =>
      tag.name === name &&
      tag.module &&
      (source === tag.module || source?.startsWith(`${tag.module}/`)),
  );
}

function literalSql(node, context, options) {
  if (node?.type === "Literal" && typeof node.value === "string") return true;
  if (node?.type === "TemplateLiteral") return node.expressions.length === 0;
  if (node?.type !== "TaggedTemplateExpression" || node.tag.type !== "Identifier") return false;
  const variable = resolveVariable(node.tag, context);
  if (variable?.references.some((reference) => reference.isWrite())) return false;
  return Boolean(
    variable?.defs.some(
      (def) =>
        def.type === "ImportBinding" &&
        def.node.type === "ImportSpecifier" &&
        def.parent.importKind !== "type" &&
        def.node.importKind !== "type" &&
        configuredTag(def.parent.source.value, def.node.imported.name, options),
    ),
  );
}

function tsLiteralSql(node, checker, ts, options) {
  if (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node)) return true;
  if (!ts.isTaggedTemplateExpression(node) || !ts.isIdentifier(node.tag)) return false;
  const symbol = checker.getSymbolAtLocation(node.tag);
  return Boolean(
    symbol?.declarations?.some((declaration) => {
      const source = declaration.parent?.parent?.parent?.moduleSpecifier?.text;
      const name = declaration.propertyName?.text ?? declaration.name?.text;
      return (
        !declaration.isTypeOnly &&
        !declaration.parent?.parent?.isTypeOnly &&
        configuredTag(source, name, options)
      );
    }),
  );
}

module.exports = { literalSql, tsLiteralSql };
