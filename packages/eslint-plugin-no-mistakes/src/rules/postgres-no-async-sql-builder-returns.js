"use strict";

const { traverse, unwrapExpression } = require("./async-ast");
const { calleeMatches, localInitializer } = require("./postgres-no-async-sql-builder-match");

// Loops, switch, and try can throw or skip a return. After `async` is removed
// that exception happens before the caller continues.
// Parentheses and assertions sit between a member callee and its tag.
const TAG_WRAPPERS = new Set([
  "ChainExpression",
  "ParenthesizedExpression",
  "TSAsExpression",
  "TSNonNullExpression",
  "TSSatisfiesExpression",
  "TSTypeAssertion",
]);

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

function isSafePlaceholder(node, builders, context, func, seen) {
  const current = unwrapExpression(node);
  if (current.type === "Literal" || isConstantTemplate(current)) return true;
  if (current.type !== "Identifier") return isJudgedReturn(current, builders, context, func, seen);
  if (seen.has(current)) return false;
  const initializer = localInitializer(current, func, context);
  if (!initializer) return true;
  seen.add(current);
  const safe = isSafePlaceholder(initializer, builders, context, func, seen);
  seen.delete(current);
  return safe;
}

function isJudgedReturn(node, builders, context, func, seen) {
  const current = unwrapExpression(node);
  if (current.type === "ConditionalExpression") {
    return (
      isJudgedReturn(current.consequent, builders, context, func, seen) &&
      isJudgedReturn(current.alternate, builders, context, func, seen)
    );
  }
  if (current.type === "LogicalExpression") {
    return (
      isJudgedReturn(current.left, builders, context, func, seen) &&
      isJudgedReturn(current.right, builders, context, func, seen)
    );
  }
  if (current.type === "SequenceExpression") {
    return isJudgedReturn(
      current.expressions[current.expressions.length - 1],
      builders,
      context,
      func,
      seen,
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
      isSafePlaceholder(expression, builders, context, func, seen),
    );
  }
  if (current.type !== "Identifier" || seen.has(current)) return false;
  const initializer = localInitializer(current, func, context);
  if (!initializer) return false;
  seen.add(current);
  return isJudgedReturn(initializer, builders, context, func, seen);
}

function statementCompletion(statement) {
  if (statement.type === "ReturnStatement" || statement.type === "ThrowStatement") return "returns";
  if (statement.type === "BlockStatement") return blockCompletion(statement.body);
  if (statement.type === "LabeledStatement") return statementCompletion(statement.body);
  if (statement.type === "IfStatement") {
    const whenTrue = statementCompletion(statement.consequent);
    if (!statement.alternate) return whenTrue === "returns" || whenTrue === "may" ? "may" : "next";
    const whenFalse = statementCompletion(statement.alternate);
    if (whenTrue === "returns" && whenFalse === "returns") return "returns";
    if (whenTrue === "falls" || whenTrue === "next") {
      return whenFalse === "falls" || whenFalse === "next" ? "next" : "may";
    }
    return "may";
  }
  if (
    statement.type === "ExpressionStatement" ||
    statement.type === "VariableDeclaration" ||
    statement.type === "FunctionDeclaration" ||
    statement.type === "ClassDeclaration" ||
    statement.type === "EmptyStatement" ||
    statement.type === "DebuggerStatement"
  ) {
    return "next";
  }
  return "may";
}

function blockCompletion(statements) {
  for (let index = 0; index < statements.length; index += 1) {
    const completion = statementCompletion(statements[index]);
    if (completion === "returns") return "returns";
    if (completion === "may") {
      return blockCompletion(statements.slice(index + 1)) === "returns" ? "returns" : "may";
    }
  }
  return "falls";
}

function returnsJudgedSql(context, func, builders) {
  if (func.body.type !== "BlockStatement") {
    return isJudgedReturn(func.body, builders, context, func, new Set());
  }
  if (blockCompletion(func.body.body) !== "returns") return false;
  let judged = true;
  let seenReturn = false;
  traverse(context, func.body, (node) => {
    if (!judged || node.type !== "ReturnStatement") return;
    seenReturn = true;
    if (!node.argument || !isJudgedReturn(node.argument, builders, context, func, new Set())) {
      judged = false;
    }
  });
  return judged && seenReturn;
}

function parameterCanThrow(param) {
  return (
    param.type === "ObjectPattern" ||
    param.type === "ArrayPattern" ||
    param.type === "AssignmentPattern"
  );
}

function isConfiguredTagMember(node, builders, context) {
  let current = node;
  // Climb the callee chain (`db.helper.sql`, `db.sql!`) and stop at the tag.
  while (
    current.parent.object === current ||
    (current.parent.expression === current && TAG_WRAPPERS.has(current.parent.type))
  ) {
    current = current.parent;
  }
  const parent = current.parent;
  return (
    parent.type === "TaggedTemplateExpression" && isConfiguredSqlCall(parent, builders, context)
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
    if (node.type === "MemberExpression" && !isConfiguredTagMember(node, builders, context)) {
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
