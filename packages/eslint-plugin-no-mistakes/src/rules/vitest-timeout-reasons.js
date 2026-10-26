"use strict";
const { adjacentAnnotations } = require("../adjacent-annotation");
function timeoutJustified(sourceCode, entry, origin, registration) {
  const targets = [registration];
  if (origin.range[0] >= registration.range[0] && origin.range[1] <= registration.range[1])
    targets.push(origin);
  // Effective property annotations can travel with options, scalar declarations cannot.
  const effective = entry.annotationOrigin || entry.origin;
  if (effective?.type === "Property") targets.push(effective, effective.value);
  return targets.some((target) =>
    adjacentAnnotations(sourceCode, target, "no-mistakes-timeout-reason:").some(
      (text) =>
        /[\p{L}\p{N}]/u.test(text) &&
        !/^(?:todo|tbd|fixme|xxx|placeholder)\b/i.test(text) &&
        !/^(?:<?reason>?|n\/a|none)$/i.test(text),
    ),
  );
}
module.exports = { timeoutJustified };
