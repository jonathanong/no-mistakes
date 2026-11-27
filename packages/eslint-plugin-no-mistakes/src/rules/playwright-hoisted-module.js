"use strict";

const { unwrapExpression } = require("./async-ast");
const { resolveModuleBinding } = require("./module-binding");
const { invokedCall, memberName } = require("./playwright-static-call");
const { importSpecifierName } = require("./test-no-shared-state-callees");

const PLAYWRIGHT_PATH_PATTERN =
  /(?:^|[/\\])(?:e2e|playwright)(?:[/\\]|$)|(?:^|[/\\])e2e\.(?:spec|test)\.[cm]?[jt]sx?$|\.pw\.(?:spec|test)\.[cm]?[jt]sx?$/;

function createHoistedModuleState(context) {
  let playwrightFile = PLAYWRIGHT_PATH_PATTERN.test(context.filename.replace(/\\/g, "/"));
  const testCalleeNames = new Set(["test", "it", "describe"]);
  const namespaces = new Set();

  function noteBinding(id) {
    if (!id) return;
    if (id.type === "ObjectPattern") {
      for (const property of id.properties) {
        if (property.type !== "Property") continue;
        const value =
          property.value?.type === "AssignmentPattern" ? property.value.left : property.value;
        noteBinding(value);
      }
      return;
    }
    if (id.type !== "Identifier") return;
    const fact = resolveModuleBinding(id, context);
    if (fact?.source !== "@playwright/test") return;
    playwrightFile = true;
    if (fact.status === "namespace") namespaces.add(id.name);
    if (
      fact.status === "export" &&
      (fact.exported === "default" || ["describe", "it", "test"].includes(fact.exported))
    ) {
      testCalleeNames.add(id.name);
    }
  }

  function admitImport(node) {
    if (node.source.value !== "@playwright/test") return;
    playwrightFile = true;
    for (const specifier of node.specifiers) {
      if (specifier.type === "ImportDefaultSpecifier" && specifier.local?.name) {
        testCalleeNames.add(specifier.local.name);
      }
      if (specifier.type === "ImportNamespaceSpecifier" && specifier.local?.name) {
        namespaces.add(specifier.local.name);
      }
      if (specifier.type !== "ImportSpecifier") continue;
      const imported = importSpecifierName(specifier);
      if (["describe", "it", "test"].includes(imported) && specifier.local?.name) {
        testCalleeNames.add(specifier.local.name);
      }
    }
  }

  function opaqueBeforeAll(node) {
    const callee = unwrapExpression(invokedCall(node)?.callee ?? node.callee);
    if (memberName(callee) !== "beforeAll") return false;
    const fact = resolveModuleBinding(unwrapExpression(callee.object), context);
    return fact?.status === "opaque";
  }

  return {
    admitImport,
    isPlaywrightFile: () => playwrightFile,
    namespaces,
    noteBinding,
    opaqueBeforeAll,
    testCalleeNames,
  };
}

module.exports = { createHoistedModuleState };
