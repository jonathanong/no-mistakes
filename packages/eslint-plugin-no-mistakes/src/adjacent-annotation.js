"use strict";

// Parentheses required by a parent construct are not grouping around the target.
function syntaxParen(sourceCode, node) {
  // ESTree wraps an optional registration without introducing any syntax.
  while (node.parent?.type === "ChainExpression") node = node.parent;
  const parent = node.parent;
  if (!parent) return null;
  if (["CallExpression", "NewExpression"].includes(parent.type)) {
    return sourceCode.getTokenAfter(
      parent.typeArguments || parent.typeParameters || parent.callee,
      {
        filter: (token) => token.value === "(",
      },
    );
  }
  if (
    [
      "IfStatement",
      "WhileStatement",
      "SwitchStatement",
      "WithStatement",
      "ImportExpression",
    ].includes(parent.type)
  )
    return sourceCode.getTokenAfter(sourceCode.getFirstToken(parent));
  if (parent.type === "DoWhileStatement")
    return sourceCode.getTokenAfter(sourceCode.getTokenAfter(parent.body));
  return null;
}

// Only comments, whitespace and grouping may separate an annotation and its target.
// A comment trailing another statement belongs to that statement, never the target.
function adjacentAnnotations(sourceCode, node, marker) {
  const comments = [];
  let left = node;
  let right = node;
  const boundary = syntaxParen(sourceCode, node);
  for (;;) {
    const previous = sourceCode.getTokenBefore(left, { includeComments: true });
    if (!previous || left.loc.start.line - previous.loc.end.line > 1) break;
    if (["Line", "Block"].includes(previous.type)) {
      comments.push(previous);
      left = previous;
      continue;
    }
    const closing = sourceCode.getTokenAfter(right);
    if (
      previous.value !== "(" ||
      closing?.value !== ")" ||
      previous.range[0] === boundary?.range[0]
    )
      break;
    left = previous;
    right = closing;
  }
  const result = [];
  for (const comment of comments) {
    const previous = sourceCode.getTokenBefore(comment);
    if (
      previous?.loc.end.line === comment.loc.start.line &&
      !["(", "[", "{", ",", ":"].includes(previous.value)
    )
      break;
    for (const text of comment.value.split(/\r?\n/)) {
      const normalized = text.replace(/^\s*\*?\s?/, "");
      if (normalized.startsWith(marker)) result.push(normalized.slice(marker.length).trim());
    }
  }
  return result;
}
module.exports = { adjacentAnnotations };
