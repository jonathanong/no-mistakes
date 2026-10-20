"use strict";

const { traverse, unwrapExpression } = require("./async-ast");
const { blockCompletion } = require("./postgres-no-async-sql-builder-completion");
const {
  moduleConstSqlDeclaration,
  moduleConstsAreReady,
} = require("./postgres-no-async-sql-builder-constants");
const {
  calleeMatches,
  localInitializer,
  tagCalleeIsStable,
} = require("./postgres-no-async-sql-builder-match");

// Parentheses and assertions sit between a member callee and its tag.
const TAG_WRAPPERS = new Set([
  "ChainExpression",
  "ParenthesizedExpression",
  "TSAsExpression",
  "TSNonNullExpression",
  "TSSatisfiesExpression",
  "TSTypeAssertion",
]);

// Loops, switch, and try can throw or skip a return. After `async` is removed
// that exception happens before the caller continues.
const UNSAFE_STATEMENTS = new Set([
  "DoWhileStatement",
  "ForInStatement",
  "ForOfStatement",
  "ForStatement",
  "SwitchStatement",
  "ThrowStatement",
  "TryStatement",
  "WhileStatement",
  "WithStatement",
]);

function isStringLiteral(node) {
  return node.type === "Literal" && typeof node.value === "string";
}

function isConstantTemplate(node) {
  return node.type === "TemplateLiteral" && node.expressions.length === 0;
}

function isConfiguredSqlCall(node, builders, context) {
  // Callers only pass a call or a tagged template.
  const callee = node.type === "CallExpression" ? node.callee : node.tag;
  return builders.some(
    (builder) => builder.callees.length > 0 && calleeMatches(callee, builder, context),
  );
}

function isSafePlaceholder(node, builders, context, func, seen, consts) {
  const current = unwrapExpression(node);
  if (current.type === "Literal" || isConstantTemplate(current)) return true;
  if (current.type !== "Identifier") {
    return isJudgedReturn(current, builders, context, func, seen, consts);
  }
  if (seen.has(current)) return false;
  const initializer = localInitializer(current, func, context);
  if (!initializer) {
    const declaration = moduleConstSqlDeclaration(current, context);
    if (declaration) consts.push(declaration);
    return true;
  }
  seen.add(current);
  const safe = isSafePlaceholder(initializer, builders, context, func, seen, consts);
  seen.delete(current);
  return safe;
}

function isJudgedReturn(node, builders, context, func, seen, consts) {
  const current = unwrapExpression(node);
  if (current.type === "ConditionalExpression") {
    return (
      isJudgedReturn(current.consequent, builders, context, func, seen, consts) &&
      isJudgedReturn(current.alternate, builders, context, func, seen, consts)
    );
  }
  if (current.type === "LogicalExpression") {
    return (
      isJudgedReturn(current.left, builders, context, func, seen, consts) &&
      isJudgedReturn(current.right, builders, context, func, seen, consts)
    );
  }
  if (current.type === "SequenceExpression") {
    return isJudgedReturn(
      current.expressions[current.expressions.length - 1],
      builders,
      context,
      func,
      seen,
      consts,
    );
  }
  if (isStringLiteral(current) || isConstantTemplate(current)) return true;
  // A call such as sql(id) is not a trusted tag. A placeholder must be an
  // identifier, a literal, or already judged SQL; id.x throws once async is gone.
  if (
    current.type === "TaggedTemplateExpression" &&
    isConfiguredSqlCall(current, builders, context)
  ) {
    return current.quasi.expressions.every((expression) =>
      isSafePlaceholder(expression, builders, context, func, seen, consts),
    );
  }
  if (current.type !== "Identifier" || seen.has(current)) return false;
  const initializer = localInitializer(current, func, context);
  if (initializer) {
    seen.add(current);
    return isJudgedReturn(initializer, builders, context, func, seen, consts);
  }
  const declaration = moduleConstSqlDeclaration(current, context);
  if (!declaration) return false;
  consts.push(declaration);
  return true;
}

function returnsJudgedSql(context, func, builders) {
  const consts = [];
  if (func.body.type !== "BlockStatement") {
    return (
      isJudgedReturn(func.body, builders, context, func, new Set(), consts) &&
      moduleConstsAreReady(func, consts, context)
    );
  }
  if (blockCompletion(func.body.body) !== "returns") return false;
  let judged = true;
  let seenReturn = false;
  traverse(context, func.body, (node) => {
    if (!judged || node.type !== "ReturnStatement") return;
    seenReturn = true;
    if (
      !node.argument ||
      !isJudgedReturn(node.argument, builders, context, func, new Set(), consts)
    ) {
      judged = false;
    }
  });
  return judged && seenReturn && moduleConstsAreReady(func, consts, context);
}

function parameterCanThrow(param) {
  return (
    param.type === "ObjectPattern" ||
    param.type === "ArrayPattern" ||
    param.type === "AssignmentPattern"
  );
}

function isConfiguredTagMember(node, builders, context, func) {
  let current = node;
  // Climb the callee chain (`db.helper.sql`, `db.sql!`) and stop at the tag.
  while (
    current.parent?.object === current ||
    (current.parent?.expression === current && TAG_WRAPPERS.has(current.parent.type))
  ) {
    current = current.parent;
  }
  const parent = current.parent;
  return (
    parent?.type === "TaggedTemplateExpression" &&
    isConfiguredSqlCall(parent, builders, context) &&
    tagCalleeIsStable(parent.tag, context, func)
  );
}

function bodyHasUnjudgedEffect(context, func, builders) {
  // Patterns and defaults run before the body and can throw once async is gone.
  if (func.params.some(parameterCanThrow)) return true;
  let unsafe = false;
  traverse(context, func.body, (node) => {
    if (unsafe) return;
    if (UNSAFE_STATEMENTS.has(node.type) || node.type === "NewExpression") {
      unsafe = true;
      return;
    }
    // A getter throws before later statements. The judged tag callee is the SQL.
    if (node.type === "MemberExpression" && !isConfiguredTagMember(node, builders, context, func)) {
      unsafe = true;
      return;
    }
    if (
      (node.type === "CallExpression" || node.type === "TaggedTemplateExpression") &&
      !isConfiguredSqlCall(node, builders, context)
    ) {
      unsafe = true;
    }
  });
  return unsafe;
}

module.exports = {
  bodyHasUnjudgedEffect,
  returnsJudgedSql,
};
