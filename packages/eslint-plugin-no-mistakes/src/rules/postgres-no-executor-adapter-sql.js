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

// `statement.append(fragment)` stays static when both sides are static SQL; a
// string argument is raw SQL, so only a literal one is accepted.
function isAppend(node) {
  return (
    node?.type === "CallExpression" &&
    node.arguments.length === 1 &&
    node.callee.type === "MemberExpression" &&
    !node.callee.computed &&
    node.callee.property.name === "append"
  );
}

function literalSql(node, context, options, safeHole) {
  if (isAppend(node))
    return [node.callee.object, node.arguments[0]].every((part) =>
      literalSql(part, context, options, safeHole),
    );
  if (node?.type === "Literal" && typeof node.value === "string") return true;
  if (node?.type === "TemplateLiteral") return node.expressions.length === 0;
  if (node?.type !== "TaggedTemplateExpression" || node.tag.type !== "Identifier") return false;
  if (!node.quasi.quasis[0]?.value.cooked?.trim()) return false;
  if (!node.quasi.expressions.every(safeHole)) return false;
  const variable = resolveVariable(node.tag, context);
  if (variable?.references.some((reference) => reference.isWrite())) return false;
  return Boolean(
    variable?.defs.some(
      (def) =>
        def.type === "ImportBinding" &&
        def.parent.importKind !== "type" &&
        def.node.importKind !== "type" &&
        ((def.node.type === "ImportDefaultSpecifier" &&
          configuredTag(def.parent.source.value, "default", options)) ||
          (def.node.type === "ImportSpecifier" &&
            configuredTag(def.parent.source.value, def.node.imported.name, options))),
    ),
  );
}

function tsLiteralSql(node, checker, ts, options, safeHole) {
  if (
    ts.isCallExpression(node) &&
    ts.isPropertyAccessExpression(node.expression) &&
    node.expression.name.text === "append" &&
    node.arguments.length === 1
  )
    return [node.expression.expression, node.arguments[0]].every((part) =>
      tsLiteralSql(part, checker, ts, options, safeHole),
    );
  if (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node)) return true;
  if (!ts.isTaggedTemplateExpression(node) || !ts.isIdentifier(node.tag)) return false;
  const head = ts.isTemplateExpression(node.template)
    ? node.template.head.text
    : node.template.text;
  if (!head.trim()) return false;
  if (
    ts.isTemplateExpression(node.template) &&
    !node.template.templateSpans.every((span) => safeHole(span.expression))
  )
    return false;
  const symbol = checker.getSymbolAtLocation(node.tag);
  return Boolean(
    symbol?.declarations?.some((declaration) => {
      if (ts.isImportClause(declaration)) {
        return (
          !declaration.isTypeOnly &&
          configuredTag(declaration.parent.moduleSpecifier.text, "default", options)
        );
      }
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
