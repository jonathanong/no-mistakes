"use strict";

function templateLiteral(node, { ts, convert, make }) {
  const spans = node.templateSpans || [];
  const parts = [node.head || node, ...spans.map((span) => span.literal)];
  return make(node, "TemplateLiteral", {
    expressions: spans.map((span) => convert(span.expression)),
    quasis: parts.map((part, index) =>
      make(part, "TemplateElement", {
        tail: index === parts.length - 1,
        value: {
          raw: part.rawText ?? part.text,
          cooked: part.templateFlags & ts.TokenFlags.ContainsInvalidEscape ? null : part.text,
        },
      }),
    ),
  });
}

module.exports = { templateLiteral };
