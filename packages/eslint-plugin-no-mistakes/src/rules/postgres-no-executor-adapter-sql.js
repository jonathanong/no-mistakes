"use strict";

const { resolveVariable } = require("./postgres-query-text");
const {
  constFragment,
  isAppend,
  tsConstFragment,
} = require("./postgres-no-executor-adapter-sql-const");

function configuredTag(source, name, options) {
  return options.trustedSqlTags.some(
    (tag) =>
      tag.name === name &&
      tag.module &&
      (source === tag.module || source?.startsWith(`${tag.module}/`)),
  );
}

function fragment(node, context, options) {
  const { scalarArgument } = require("./postgres-no-executor-adapter-sql-scalar");
  const hole = (value) => scalarArgument(value, context);
  return constFragment(node, context, {
    local: (init) => literalSql(init, context, options, hole),
    ts:
      ({ checker, ts }) =>
      (init) =>
        tsLiteralSql(init, checker, ts, options, () => false),
  });
}

function literalSql(node, context, options, safeHole) {
  if (isAppend(node))
    return (
      literalSql(node.callee.object, context, options, safeHole) &&
      (literalSql(node.arguments[0], context, options, safeHole) ||
        fragment(node.arguments[0], context, options))
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

function tsLiteralSql(node, checker, ts, options, safeHole, program) {
  if (
    ts.isCallExpression(node) &&
    ts.isPropertyAccessExpression(node.expression) &&
    node.expression.name.text === "append" &&
    node.arguments.length === 1
  ) {
    const part = (value) => tsLiteralSql(value, checker, ts, options, safeHole, program);
    const judge = (init) => tsLiteralSql(init, checker, ts, options, () => false);
    return (
      part(node.expression.expression) &&
      (part(node.arguments[0]) ||
        tsConstFragment(node.arguments[0], { checker, ts, program }, judge))
    );
  }
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
