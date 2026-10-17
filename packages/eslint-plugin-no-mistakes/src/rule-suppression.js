"use strict";

// Parse actual comments once; directive-looking string literals never suppress.
// requireReason is opt-in. Callers that omit it keep today's directive match.
function ruleSuppression(context, ruleId, suppressionOptions) {
  const lines = new Set();
  let file = false;
  const source = context.sourceCode;
  const first = source.ast.tokens[0];
  const requireReason = suppressionOptions?.requireReason === true;
  for (const comment of source.getAllComments()) {
    for (const [offset, text] of comment.value.split(/\r?\n/).entries()) {
      const match = /^\s*\*?\s*no-mistakes-disable-(file|line|next-line)\b(.*)$/.exec(text);
      if (!match) continue;
      const names = match[2]
        .split("--")[0]
        .trim()
        .split(/[\s,]+/)
        .filter(Boolean);
      if (
        names.length &&
        !names.some((name) => name === ruleId || name === `no-mistakes/${ruleId}`)
      )
        continue;
      if (requireReason && !directiveReason(match[2])) continue;
      if (match[1] === "file") {
        if (!first || comment.range[1] <= first.range[0]) file = true;
      } else {
        lines.add(comment.loc.start.line + offset + (match[1] === "next-line" ? 1 : 0));
      }
    }
  }
  return (node) => file || lines.has(node.loc.start.line);
}

function directiveReason(tail) {
  const marker = tail.indexOf("--");
  if (marker === -1) return "";
  return tail.slice(marker + 2).trim();
}

module.exports = { ruleSuppression };
