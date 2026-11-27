"use strict";

const { literalString, rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { unwrapExpression } = require("./async-ast");
const { admitsPlaywrightModule } = require("./module-binding");
const { childNodes } = require("./test-no-shared-state-helpers");
const { invokedCall, memberName, methodFromCallee } = require("./playwright-static-call");
const { isSentinelScrollCall } = require("./playwright-sentinel-scroll");
const { isRawScrollCall } = require("./playwright-scroll-call");

const PLAYWRIGHT_PATH_PATTERN =
  /(?:^|[/\\])(?:e2e|playwright)(?:[/\\]|$)|(?:^|[/\\])e2e\.(?:spec|test)\.[cm]?[jt]sx?$|\.pw\.(?:spec|test)\.[cm]?[jt]sx?$/;

const CURSOR_WAIT_PROPERTIES = new Set(["waitForRequest", "waitForResponse"]);
const SEARCH_PARAMS_ACCESSORS = new Set(["has", "get", "getAll"]);

// The TypeScript node types that wrap a runtime value expression rather than being purely
// type-positioned; every other "TS"-prefixed node type is skipped by `collectLiteralStrings` — a
// string literal appearing only inside a type alias or annotation is never evaluated at runtime
// and must not be mistaken for an actual cursor-param check.
const TS_VALUE_WRAPPER_TYPES = new Set([
  "TSAsExpression",
  "TSSatisfiesExpression",
  "TSNonNullExpression",
  "TSInstantiationExpression",
  "TSTypeAssertion",
]);

function isPlaywrightPath(filename) {
  return PLAYWRIGHT_PATH_PATTERN.test(filename.replace(/\\/g, "/"));
}

function isCursorWaitCall(node, context) {
  const invoked = invokedCall(node);
  if (!invoked) return null;
  return CURSOR_WAIT_PROPERTIES.has(methodFromCallee(invoked.callee, context)) ? invoked : null;
}

// `<something>.searchParams.has("after")` / `.get("after")` / `.getAll("after")` — the cursor
// param name is passed as a bare accessor argument, never appearing with a trailing `=` the way
// it does in a raw query string, so it needs its own literal-collection path rather than the
// boundary-matched one below.
function searchParamsAccessorCall(node) {
  const invoked = invokedCall(node);
  const callee = invoked?.callee;
  const object = callee;
  if (object?.type !== "MemberExpression" || !SEARCH_PARAMS_ACCESSORS.has(memberName(object))) {
    return null;
  }
  let receiver = unwrapExpression(object.object);
  if (receiver?.type !== "MemberExpression" || memberName(receiver) !== "searchParams") return null;
  return invoked;
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

// Regex-derived literals are matched as pattern *source text*, not executed, so a regex author's
// own boundary syntax (`[?&]after=`, `(?:^|[?&])after=`) leaves a `]` or `)` immediately before
// the param name instead of a literal `?`/`&`. Only these exact, known boundary constructs count
// as a separator for a regex-derived literal — accepting an arbitrary `)`/`]` before the param
// would also match an unrelated alternation like `(?:next|prev)cursor=`, which closes a group
// that has nothing to do with a query-string separator. A bare `&`, an escaped `\?`, or a `^`
// start anchor written directly (`/&after=/`, `/\?after=/`, `/^after=/`) is unambiguous — unlike
// `)`/`]`, none of these three characters ever closes an unrelated construct — so they count too.
const REGEX_BOUNDARY_SUFFIXES = [
  "(?:^|[?&])",
  "(?:^|[&?])",
  "(?:^|\\?|&)",
  "(?:^|&|\\?)",
  "[?&]",
  "[&?]",
  "&",
  "\\?",
  "^",
];

// Requires the cursor param to appear as an actual query-key boundary (`?after=`, `&after=`, or
// `after=` at the very start of the literal) rather than an unconstrained substring match, which
// would false-positive on e.g. `category=after-hours` for a configured param of `after`.
function hasQueryParamBoundary(literal, param, isRegex) {
  const target = new RegExp(`${escapeRegExp(param)}=`, "g");
  for (const match of literal.matchAll(target)) {
    const prefix = literal.slice(0, match.index);
    if (prefix.length === 0) return true;
    if (isRegex) {
      if (REGEX_BOUNDARY_SUFFIXES.some((suffix) => prefix.endsWith(suffix))) return true;
    } else if (/[?&]$/.test(prefix)) {
      return true;
    }
  }
  return false;
}

// TypeScript type-only positions (a type alias, an interface body, a `: typeof x` annotation, ...)
// are skipped entirely — a string literal type or template literal type is never evaluated at
// runtime and must not be mistaken for an actual cursor-param check. The value-wrapper expressions
// above recurse into their runtime `.expression` only, never a type operand.
function collectLiteralStrings(node, results) {
  if (node.type.startsWith("TS")) {
    if (TS_VALUE_WRAPPER_TYPES.has(node.type)) collectLiteralStrings(node.expression, results);
    return;
  }
  if (node.type === "Literal") {
    if (typeof node.value === "string") results.push({ value: node.value, isRegex: false });
    else if (node.regex) results.push({ value: node.regex.pattern, isRegex: true });
  } else if (node.type === "TemplateElement") {
    results.push({ value: node.value?.raw ?? "", isRegex: false });
  }
  for (const child of childNodes(node)) collectLiteralStrings(child, results);
}

// A `.searchParams.has()`/`.get()` argument is never boundary-matched (see above), so an
// interpolation-free template literal (`` `cursor` ``) is just as statically knowable as a plain
// string literal — only a template with actual `${...}` interpolation is genuinely dynamic.
function staticStringValue(node) {
  return literalString(node);
}

function collectSearchParamsAccessorArgs(node, results) {
  if (node.type === "CallExpression") {
    const invoked = searchParamsAccessorCall(node);
    const value = invoked ? staticStringValue(invoked.arguments[0]) : null;
    if (value !== null) results.push(value);
  }
  for (const child of childNodes(node)) collectSearchParamsAccessorArgs(child, results);
}

function mentionsCursorParam(node, cursorParams) {
  if (!node) return false;
  const literals = [];
  collectLiteralStrings(node, literals);
  if (
    literals.some(({ value, isRegex }) =>
      cursorParams.some((param) => hasQueryParamBoundary(value, param, isRegex)),
    )
  ) {
    return true;
  }
  const accessorArgs = [];
  collectSearchParamsAccessorArgs(node, accessorArgs);
  return accessorArgs.some((value) => cursorParams.includes(value));
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "disallow one-shot scroll calls that can miss infinite-scroll pagination",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        properties: {
          cursorParams: { type: "array", items: { type: "string" } },
          scrollHelper: { type: "string" },
          sentinelLocatorNames: { type: "array", items: { type: "string" } },
          sentinelTestIds: { type: "array", items: { type: "string" } },
        },
      },
    ],
    messages: {
      sentinelScroll:
        "scrollIntoViewIfNeeded() may leave this pagination sentinel stationary when it is already visible.{{helperHint}}",
      rawScroll:
        "This file awaits a cursor-paginated request/response but drives scrolling with a raw scrollTo/scroll/scrollBy call — a single synthetic scroll can land before a deferred IntersectionObserver mounts and be lost forever, stalling the wait for its full timeout.{{helperHint}}",
    },
  },
  (context) => {
    let isPlaywrightFile = isPlaywrightPath(context.filename);
    const options = context.options?.[0] ?? {};
    const cursorParams = options.cursorParams ?? ["after", "cursor"];
    const sentinelLocatorNames = new Set(options.sentinelLocatorNames);
    const sentinelTestIds = new Set(options.sentinelTestIds);
    const checkSentinels = sentinelLocatorNames.size > 0 || sentinelTestIds.size > 0;
    const scrollHelper = options.scrollHelper ?? "";
    const helperHint = scrollHelper
      ? ` Use ${scrollHelper}, which scrolls repeatedly until the request fires, instead.`
      : " Use a helper that scrolls repeatedly until the request fires instead of a single raw call.";
    const suppressed = ruleSuppression(context, "playwright-no-raw-scroll-pagination");
    let hasCursorWait = false;
    const scrollCandidates = [];
    const sentinelCandidates = [];

    return {
      ImportDeclaration(node) {
        if (admitsPlaywrightModule(node, context)) isPlaywrightFile = true;
      },
      TSImportEqualsDeclaration(node) {
        if (admitsPlaywrightModule(node, context)) isPlaywrightFile = true;
      },
      VariableDeclarator(node) {
        if (admitsPlaywrightModule(node, context)) isPlaywrightFile = true;
      },
      ExpressionStatement(node) {
        if (admitsPlaywrightModule(node, context)) isPlaywrightFile = true;
      },
      CallExpression(node) {
        const wait = isCursorWaitCall(node, context);
        if (wait && mentionsCursorParam(wait.arguments[0], cursorParams)) {
          hasCursorWait = true;
        }
        if (isRawScrollCall(node, context)) scrollCandidates.push(node);
        if (
          checkSentinels &&
          isSentinelScrollCall(node, context, sentinelLocatorNames, sentinelTestIds)
        ) {
          sentinelCandidates.push(node);
        }
      },
      "Program:exit"() {
        if (!isPlaywrightFile) return;
        for (const node of sentinelCandidates) {
          if (suppressed(node)) continue;
          context.report({ node, messageId: "sentinelScroll", data: { helperHint } });
        }
        if (!hasCursorWait) return;
        for (const node of scrollCandidates) {
          if (suppressed(node)) continue;
          context.report({ node, messageId: "rawScroll", data: { helperHint } });
        }
      },
    };
  },
);
