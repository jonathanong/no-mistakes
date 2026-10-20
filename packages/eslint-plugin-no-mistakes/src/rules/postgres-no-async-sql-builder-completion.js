"use strict";

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

module.exports = {
  blockCompletion,
};
