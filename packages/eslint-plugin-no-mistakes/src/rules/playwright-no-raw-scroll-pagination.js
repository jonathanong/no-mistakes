"use strict";

const { rule } = require("../helpers");
const { childNodes } = require("./test-no-shared-state-helpers");

const PLAYWRIGHT_PATH_PATTERN =
  /(?:^|[/\\])(?:e2e|playwright)(?:[/\\]|$)|(?:^|[/\\])e2e\.(?:spec|test)\.[cm]?[jt]sx?$|\.pw\.(?:spec|test)\.[cm]?[jt]sx?$/;

const CURSOR_WAIT_PROPERTIES = new Set(["waitForRequest", "waitForResponse"]);
const RAW_SCROLL_NAMES = new Set(["scrollTo", "scrollBy"]);

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

// `window.scrollTo(...)`, bare `scrollTo(...)`, and the `scrollBy` equivalents — the only two
// imperative, position-based browser scroll APIs. `scrollIntoView` is element-relative, not a
// pagination driver, and is intentionally not matched.
function isRawScrollCall(node) {
  const callee = node.callee;
  if (callee.type === "Identifier") return RAW_SCROLL_NAMES.has(callee.name);
  return (
    callee.type === "MemberExpression" &&
    !callee.computed &&
    RAW_SCROLL_NAMES.has(propertyName(callee.property))
  );
}

function collectLiteralStrings(node, results) {
  if (!node) return;
  if (node.type === "Literal") {
    if (typeof node.value === "string") results.push(node.value);
    else if (node.regex) results.push(node.regex.pattern);
  } else if (node.type === "TemplateElement") {
    results.push(node.value?.raw ?? "");
  }
  for (const child of childNodes(node)) collectLiteralStrings(child, results);
}

function mentionsCursorParam(node, cursorParams) {
  if (!node) return false;
  const literals = [];
  collectLiteralStrings(node, literals);
  return literals.some((literal) => cursorParams.some((param) => literal.includes(param)));
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description:
        "disallow driving cursor-paginated infinite scroll with a raw window.scrollTo/scrollBy",
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
        "This file awaits a cursor-paginated request/response but drives scrolling with a raw scrollTo/scrollBy call — a single synthetic scroll can land before a deferred IntersectionObserver mounts and be lost forever, stalling the wait for its full timeout.{{helperHint}}",
    },
  },
  (context) => {
    if (!isPlaywrightPath(context.filename)) return {};
    const options = context.options?.[0] ?? {};
    const cursorParams = options.cursorParams ?? ["after", "cursor"];
    const scrollHelper = options.scrollHelper ?? "";
    const helperHint = scrollHelper
      ? ` Use ${scrollHelper}, which scrolls repeatedly until the request fires, instead.`
      : " Use a helper that scrolls repeatedly until the request fires instead of a single raw call.";
    let hasCursorWait = false;
    const scrollCandidates = [];

    return {
      CallExpression(node) {
        if (isCursorWaitCall(node) && mentionsCursorParam(node.arguments[0], cursorParams)) {
          hasCursorWait = true;
        }
        if (isRawScrollCall(node)) scrollCandidates.push(node);
      },
      "Program:exit"() {
        if (!hasCursorWait) return;
        for (const node of scrollCandidates) {
          context.report({ node, messageId: "rawScroll", data: { helperHint } });
        }
      },
    };
  },
);
