"use strict";

const { rule } = require("../helpers");
const { childNodes } = require("./test-no-shared-state-helpers");
const { resolveVariable } = require("./test-no-shared-state-aliases");

const PLAYWRIGHT_PATH_PATTERN =
  /(?:^|[/\\])(?:e2e|playwright)(?:[/\\]|$)|(?:^|[/\\])e2e\.(?:spec|test)\.[cm]?[jt]sx?$|\.pw\.(?:spec|test)\.[cm]?[jt]sx?$/;

const CURSOR_WAIT_PROPERTIES = new Set(["waitForRequest", "waitForResponse"]);
const RAW_SCROLL_NAMES = new Set(["scrollTo", "scrollBy", "scroll"]);
const SEARCH_PARAMS_ACCESSORS = new Set(["has", "get"]);

function isPlaywrightPath(filename) {
  return PLAYWRIGHT_PATH_PATTERN.test(filename.replace(/\\/g, "/"));
}

function propertyName(node) {
  if (!node) return null;
  return node.type === "Literal" ? String(node.value) : node.name;
}

// `page.waitForRequest(...)` / `page.waitForResponse(...)`, matched by property name only — the
// object (page, frame, a locator, ...) doesn't matter, mirroring how `playwright-no-set-timeout`
// matches `.waitForTimeout` regardless of receiver.
function isCursorWaitCall(node) {
  return (
    node.callee.type === "MemberExpression" &&
    !node.callee.computed &&
    CURSOR_WAIT_PROPERTIES.has(propertyName(node.callee.property))
  );
}

// `window.scrollTo(...)`/`window.scroll(...)`, bare `scrollTo(...)`/`scroll(...)`, and the
// `scrollBy` equivalents — the only imperative, position-based browser scroll APIs. A same-named
// method on any other receiver (`map.scrollTo()`, an editor or page-object helper) is not a
// browser scroll and is never matched. `scrollIntoView` is element-relative, not a pagination
// driver, and is intentionally not matched. A bare call only counts when it resolves to the
// global — a project's own locally-declared or imported `scrollTo`/`scroll`/`scrollBy` helper is a
// different function and must not be misflagged.
function isRawScrollCall(node, context) {
  const callee = node.callee;
  if (callee.type === "Identifier") {
    if (!RAW_SCROLL_NAMES.has(callee.name)) return false;
    const variable = resolveVariable(callee, context);
    return !(variable?.defs?.length > 0);
  }
  return (
    callee.type === "MemberExpression" &&
    (!callee.computed || callee.property.type === "Literal") &&
    callee.object.type === "Identifier" &&
    callee.object.name === "window" &&
    RAW_SCROLL_NAMES.has(propertyName(callee.property))
  );
}

// `<something>.searchParams.has("after")` / `.get("after")` — the cursor param name is passed as
// a bare accessor argument, never appearing with a trailing `=` the way it does in a raw query
// string, so it needs its own literal-collection path rather than the boundary-matched one below.
function isSearchParamsAccessorCall(node) {
  return Boolean(
    node.callee.type === "MemberExpression" &&
    !node.callee.computed &&
    SEARCH_PARAMS_ACCESSORS.has(propertyName(node.callee.property)) &&
    node.callee.object.type === "MemberExpression" &&
    !node.callee.object.computed &&
    propertyName(node.callee.object.property) === "searchParams",
  );
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

function collectLiteralStrings(node, results) {
  if (!node) return;
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
  if (node?.type === "Literal" && typeof node.value === "string") return node.value;
  if (node?.type === "TemplateLiteral" && node.expressions.length === 0) {
    return node.quasis[0].value.raw;
  }
  return null;
}

function collectSearchParamsAccessorArgs(node, results) {
  if (!node) return;
  if (node.type === "CallExpression" && isSearchParamsAccessorCall(node)) {
    const value = staticStringValue(node.arguments[0]);
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
      description:
        "disallow driving cursor-paginated infinite scroll with a raw window.scrollTo/scroll/scrollBy",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        properties: {
          cursorParams: { type: "array", items: { type: "string" } },
          scrollHelper: { type: "string" },
        },
      },
    ],
    messages: {
      rawScroll:
        "This file awaits a cursor-paginated request/response but drives scrolling with a raw scrollTo/scroll/scrollBy call — a single synthetic scroll can land before a deferred IntersectionObserver mounts and be lost forever, stalling the wait for its full timeout.{{helperHint}}",
    },
  },
  (context) => {
    let isPlaywrightFile = isPlaywrightPath(context.filename);
    const options = context.options?.[0] ?? {};
    const cursorParams = options.cursorParams ?? ["after", "cursor"];
    const scrollHelper = options.scrollHelper ?? "";
    const helperHint = scrollHelper
      ? ` Use ${scrollHelper}, which scrolls repeatedly until the request fires, instead.`
      : " Use a helper that scrolls repeatedly until the request fires instead of a single raw call.";
    let hasCursorWait = false;
    const scrollCandidates = [];

    return {
      ImportDeclaration(node) {
        if (node.source.value === "@playwright/test") isPlaywrightFile = true;
      },
      CallExpression(node) {
        if (isCursorWaitCall(node) && mentionsCursorParam(node.arguments[0], cursorParams)) {
          hasCursorWait = true;
        }
        if (isRawScrollCall(node, context)) scrollCandidates.push(node);
      },
      "Program:exit"() {
        if (!isPlaywrightFile || !hasCursorWait) return;
        for (const node of scrollCandidates) {
          context.report({ node, messageId: "rawScroll", data: { helperHint } });
        }
      },
    };
  },
);
