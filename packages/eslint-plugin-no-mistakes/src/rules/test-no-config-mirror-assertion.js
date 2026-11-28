"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { isFunction, unwrapExpression } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { bindingResolver, isFilesystemMock } = require("./test-no-fixed-temp-paths-bindings");
const {
  createPathResolver,
  normalizeFixtureDirs,
  pathVariable,
} = require("./test-no-config-mirror-paths");
const {
  effectiveExemptionScope,
  enclosingSuiteChain,
  isExemptInScope,
  recordScopePath,
} = require("./test-no-config-mirror-scopes");
const {
  classifyExpectCall,
  collectExecutedArguments,
  writePathArgument,
} = require("./test-no-config-mirror-sinks");
const { collectSources } = require("./test-no-config-mirror-sources");
const { analyzeSourceUsage } = require("./test-no-config-mirror-flow");

const RULE_NAME = "test-no-config-mirror-assertion";
const TEST_FILE_PATTERN = /\.(?:test|spec)\.[cm]?[jt]sx?$/;

const EXTRA_MODULES = ["fs-extra", "yaml", "js-yaml", "url", "util"];
const MOCK_EXTRA_MODULES = new Set(["fs-extra"]);

function buildConfiguredParsers(parsers) {
  const extraModules = new Set(EXTRA_MODULES);
  const parserBindings = new Set();
  for (const item of parsers ?? []) {
    extraModules.add(item.module);
    parserBindings.add(item.name === "default" ? item.module : `${item.module}.${item.name}`);
  }
  return { extraModules, parserBindings };
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description:
        "disallow tests that read a tracked non-fixture file only to assert literals in its text",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        properties: {
          fixtureDirs: { type: "array", items: { type: "string", minLength: 1 } },
          parsers: {
            type: "array",
            items: {
              type: "object",
              properties: {
                module: { type: "string", minLength: 1 },
                name: { type: "string", minLength: 1 },
              },
              required: ["module", "name"],
              additionalProperties: false,
            },
          },
        },
        additionalProperties: false,
      },
    ],
    messages: {
      configMirror:
        "Avoid asserting literal values read from a tracked repository file. Delete this mirror test, enforce the invariant with a static rule, or assert a cross-file contract instead.",
    },
  },
  (context) => {
    if (!TEST_FILE_PATTERN.test(context.filename)) return {};
    const option = context.options[0] ?? {};
    const fixtureDirs = normalizeFixtureDirs(option.fixtureDirs);
    const { extraModules, parserBindings } = buildConfiguredParsers(option.parsers);
    const suppressed = ruleSuppression(context, RULE_NAME, { requireReason: true });
    const resolve = bindingResolver(context, extraModules);
    const resolvePath = createPathResolver(context, resolve, fixtureDirs);

    let mocksFilesystem = false;
    const promisifiedVars = new Set();
    const helperCandidates = [];
    const calls = [];
    const expectMap = new Map();
    const exemptPaths = new Map();

    return {
      VariableDeclarator(node) {
        if (node.id.type !== "Identifier" || !node.init) return;
        const init = unwrapExpression(node.init);
        if (
          init?.type === "CallExpression" &&
          resolve(init.callee) === "util.promisify" &&
          resolve(init.arguments[0]) === "fs.readFile"
        ) {
          promisifiedVars.add(resolveVariable(node.id, context));
        } else if (isFunction(init)) {
          helperCandidates.push({ id: node.id, fn: init });
        }
      },
      FunctionDeclaration(node) {
        helperCandidates.push({ id: node.id, fn: node });
      },
      CallExpression(node) {
        if (mocksFilesystem) return;
        const binding = resolve(node.callee);
        if (isFilesystemMock(node, binding, MOCK_EXTRA_MODULES)) {
          mocksFilesystem = true;
          return;
        }
        calls.push(node);
        const classified = classifyExpectCall(node, context);
        if (classified) expectMap.set(classified.expectCall, classified);
        const scope = effectiveExemptionScope(node, context);
        const writeArg = writePathArgument(node, binding);
        if (writeArg) {
          recordScopePath(
            exemptPaths,
            scope,
            resolvePath(writeArg),
            pathVariable(writeArg, context),
          );
        }
        collectExecutedArguments(node, binding, context, (arg) => {
          recordScopePath(exemptPaths, scope, resolvePath(arg), pathVariable(arg, context));
        });
      },
      "Program:exit"() {
        if (mocksFilesystem) return;
        const sources = collectSources({
          calls,
          helperCandidates,
          resolve,
          resolvePath,
          promisifiedVars,
          parserBindings,
          context,
        });
        const exemptTest = (block, source) =>
          isExemptInScope(exemptPaths, enclosingSuiteChain(block ?? source.node), source);
        for (const source of sources) {
          for (const finding of analyzeSourceUsage({
            source,
            context,
            resolve,
            configuredParsers: parserBindings,
            expectMap,
            exemptTest,
          })) {
            if (suppressed(finding.node) || suppressed(source.node) || suppressed(source.stmt)) {
              continue;
            }
            context.report({ node: finding.node, messageId: "configMirror" });
          }
        }
      },
    };
  },
);
