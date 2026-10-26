"use strict";

// Only whitespace may separate an annotation and its target. A comment trailing
// another statement belongs to that statement, never to the following target.
function adjacentAnnotations(sourceCode, node, marker) {
  const comments = sourceCode.getCommentsBefore(node);
  const result = [];
  let line = node.loc.start.line;
  for (const comment of comments.toReversed()) {
    if (line - comment.loc.end.line > 1) break;
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
    line = comment.loc.start.line;
  }
  return result;
}
module.exports = { adjacentAnnotations };
