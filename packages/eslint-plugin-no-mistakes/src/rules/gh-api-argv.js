"use strict";
const { unwrapExpression } = require("./async-ast");

// Unknown template substitutions are separators, never invented flag bytes.
function token(node) {
  node = unwrapExpression(node);
  if (node?.type === "Literal" && typeof node.value === "string") return node.value;
  if (node?.type === "TemplateLiteral") {
    return node.quasis.map((part) => part.value.cooked ?? part.value.raw).join("\0");
  }
  return null;
}
const fields = /^(?:-[fF].+|--(?:field|raw-field)=.*)$/;
const fieldFlags = new Set(["-f", "-F", "--field", "--raw-field"]);
const valueFlags = new Set([
  "-X",
  "--method",
  "-f",
  "-F",
  "--field",
  "--raw-field",
  "-q",
  "--jq",
  "-t",
  "--template",
  "-H",
  "--header",
  "--hostname",
  "--input",
  "--preview",
  "--cache",
]);
const switches = new Set([
  "--paginate",
  "--slurp",
  "--silent",
  "--verbose",
  "--include",
  "-i",
  "--help",
  "--exit-status",
]);
function method(value) {
  return typeof value === "string" && /^[A-Za-z]+$/.test(value);
}
function inspectArgv(array, includesApi) {
  if (array?.type !== "ArrayExpression") return null;
  const args = array.elements;
  let start = 0;
  if (includesApi) {
    if (token(args[0]) !== "api") return null;
    start = 1;
  }
  let endpoint = null;
  let hasFields = false;
  let hasMethod = false;
  let uncertain = false;
  for (let i = start; i < args.length; i += 1) {
    const text = token(args[i]);
    if (text === null) {
      uncertain = true;
      continue;
    }
    if (text.startsWith("-") && text.includes("\0")) uncertain = true;
    if (fieldFlags.has(text)) hasFields = true;
    if (fields.test(text)) hasFields = true;
    if (text === "-X" || text === "--method") hasMethod = method(token(args[i + 1]));
    if (text.startsWith("-X") && text !== "-X") hasMethod = method(text.slice(2));
    if (text.startsWith("--method=")) hasMethod = method(text.slice(9));
    if (valueFlags.has(text)) {
      if (token(args[i + 1]) === null) uncertain = true;
      const next = token(args[i + 1]);
      if (!(text === "-X" || text === "--method") || !next?.startsWith("-")) i += 1;
    } else if (
      switches.has(text) ||
      /^(?:-[fFXqHt].+)/.test(text) ||
      (text.includes("=") && valueFlags.has(text.split("=", 1)[0]))
    ) {
      continue;
    } else if (text.startsWith("-")) {
      uncertain = true;
    } else if (endpoint === null) {
      if (uncertain) return null;
      endpoint = { node: args[i], text };
    }
  }
  if (!endpoint) return null;
  // A fixed leading REST prefix plus a fixed slash cannot become a flag or
  // the exact GraphQL endpoint, even when another segment is interpolated.
  const knownRest = !endpoint.text.startsWith("\0") && endpoint.text.includes("/");
  return {
    endpoint: /\/[^?]*\?/.test(endpoint.text) ? endpoint.node : null,
    missingMethod:
      hasFields &&
      !hasMethod &&
      !uncertain &&
      (!endpoint.text.includes("\0") || knownRest) &&
      endpoint.text !== "graphql",
  };
}
module.exports = { inspectArgv, token };
