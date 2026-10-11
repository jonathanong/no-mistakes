"use strict";
const { rule, options } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { findContainingFunction, isUnconditionalBeforeReturn } = require("./async-ast");
const { createEvaluator } = require("./vitest-timeout-values");
const { createBindings } = require("./playwright-test-timeout-bindings");
const { createCarriers } = require("./playwright-timeout-carriers");

module.exports = rule(require("./playwright-timeout-meta"), (context) => {
  const option = options(context),
    max = option.max ?? 30000,
    fixtureMax = option.fixtureMax ?? max;
  for (const [key, value] of [
    ["max", max],
    ["fixtureMax", fixtureMax],
  ])
    if (!Number.isFinite(value) || value <= 0)
      throw new Error(`playwright-test-timeout-cap ${key} must be positive and finite`);
  for (const [source, roles] of Object.entries(option.exportRoles ?? {})) {
    if (source === "@playwright/test" && Object.keys(roles).length)
      throw new Error(
        "playwright-test-timeout-cap exportRoles cannot override builtin SDK exports",
      );
    if (["test", "defineConfig", "mergeTests"].some((key) => Object.hasOwn(roles, key)))
      throw new Error(
        "playwright-test-timeout-cap exportRoles cannot override recognized registrar exports",
      );
  }
  const suppressed = ruleSuppression(context, "playwright-test-timeout-cap");
  const calls = [],
    exports = [],
    states = new Map();
  const bindings = createBindings(
    context,
    new Set(["@playwright/test", ...(option.registrationPackages ?? [])]),
    option.exportRoles,
    calls,
  );
  const evaluate = createEvaluator(context, new Set());
  function report(node, messageId, name, cap = max) {
    if (!suppressed(node)) context.report({ node, messageId, data: { name, max: cap } });
  }
  function check(value, node, name, cap = max) {
    if (value.kind !== "number") {
      if (option.unknownValues === "finding") report(node, "unknown", name, cap);
    } else if (!Number.isFinite(value.value) || value.value <= 0)
      report(node, "invalid", name, cap);
    else if (value.value > cap) report(node, "timeout", name, cap);
  }
  const { field, configRoots, fixtures, slowCondition } = createCarriers(
    context,
    evaluate,
    check,
    max,
    fixtureMax,
  );
  function branchFree(node, fn) {
    return fn?.body.type === "BlockStatement" && isUnconditionalBeforeReturn(node, fn.body);
  }
  return {
    CallExpression(node) {
      calls.push(node);
    },
    ExportDefaultDeclaration(node) {
      exports.push(node.declaration);
    },
    "Program:exit"() {
      configRoots(option, exports, calls, bindings);
      const priorSlowMayExist = calls.some((node) => {
        const api = bindings.resolve(node.callee);
        if (api?.kind !== "method") return false;
        if (bindings.hooks.has(api.key) || api.key === "use" || api.key === null) return true;
        const fn = findContainingFunction(node),
          parentApi =
            fn?.parent?.type === "CallExpression" ? bindings.resolve(fn.parent.callee) : null;
        return (
          api.key === "slow" &&
          (!fn || parentApi?.key === "describe" || parentApi?.owner?.path?.includes("describe"))
        );
      });
      function initialState(fn) {
        const registrations = fn ? bindings.registrationFor(fn) : [];
        const api = registrations.length === 1 ? bindings.resolve(registrations[0].callee) : null;
        const root = api?.kind === "test" ? api.root : api?.owner?.root;
        const directSdk = root?.defs?.some(
          (def) => def.type === "ImportBinding" && def.parent.source.value === "@playwright/test",
        );
        return {
          timeout: { kind: "unknown" },
          slow: directSdk && !priorSlowMayExist ? false : null,
        };
      }
      for (const node of calls.sort((a, b) => a.range[0] - b.range[0])) {
        const api = bindings.resolve(node.callee);
        if (bindings.registration(api) && option.unknownValues === "finding")
          report(node, "effectiveSlot", "inherited effective deadline");
        if (bindings.registration(api))
          for (const argument of node.arguments)
            if (bindings.opaqueCallback(argument))
              check({ kind: "unknown" }, argument, "bound callback parameter carrier");
        if (api?.kind === "mergeTests") {
          for (const argument of node.arguments)
            if (bindings.resolve(argument)?.kind !== "test")
              check({ kind: "unknown" }, argument, "merged registrar carrier");
          continue;
        }

        if (api?.kind === "opaque") {
          check({ kind: "unknown" }, node, "admitted registrar carrier");
          continue;
        }
        if (api?.kind === "partial") {
          check({ kind: "unknown" }, node, "partially bound framework deadline");
          const state = states.get(findContainingFunction(node));
          if (state) {
            state.timeout = { kind: "unknown" };
            state.slow = null;
          }
          continue;
        }
        if (api?.kind !== "method") {
          // A helper call may mutate the active slot or slow latch. Do not invent effects.
          const state = states.get(findContainingFunction(node));
          if (state) {
            state.timeout = { kind: "unknown" };
            state.slow = null;
          }
          continue;
        }
        const owner = api.owner;
        if (api.key === null) {
          check({ kind: "unknown" }, node, "dynamic framework API");
          continue;
        }
        if (owner.kind === "test" && api.key === "extend") {
          fixtures(node.arguments[0]);
          continue;
        }
        if (owner.kind === "test" && api.key === "configure" && owner.path.includes("describe")) {
          field(evaluate(node.arguments[0]), node.arguments[0], "suite timeout");
          continue;
        }
        if (api.key !== "setTimeout" && api.key !== "slow") {
          // SDK info retrieves TestInfo; step titles/locations do not replace the test slot.
          if (
            owner.kind === "test" &&
            !bindings.registration(api) &&
            !["info", "step"].includes(api.key)
          )
            check({ kind: "unknown" }, node, "admitted framework API carrier");
          continue;
        }
        const fn = findContainingFunction(node);
        const admitted = fn && bindings.registrationFor(fn).length === 1;
        const state = states.get(fn) ?? initialState(fn);
        const ordered = admitted && branchFree(node, fn);
        if (api.key === "setTimeout") {
          const value = evaluate(node.arguments[0]);
          check(value, node.arguments[0] ?? node, "runtime timeout");
          if (ordered) state.timeout = value;
          else state.timeout = { kind: "unknown" };
        } else {
          const condition = slowCondition(node);
          if (condition === false) continue;
          if (condition === null || !ordered) {
            check({ kind: "unknown" }, node, "slow effective timeout");
            state.timeout = { kind: "unknown" };
            state.slow = null;
          } else if (state.slow === null) {
            // Unknown inherited latch: multiplication is a safe upper bound, not proof it happened.
            const upper = state.timeout.kind === "number" ? state.timeout.value * 3 : null;
            if (!(upper > 0 && Number.isFinite(upper) && upper <= max))
              check({ kind: "unknown" }, node, "slow effective timeout");
            state.timeout =
              upper > 0 && Number.isFinite(upper) && upper <= max
                ? { kind: "number", value: upper }
                : { kind: "unknown" };
            state.slow = true;
          } else if (!state.slow) {
            state.timeout =
              state.timeout.kind === "number"
                ? { kind: "number", value: state.timeout.value * 3 }
                : { kind: "unknown" };
            check(state.timeout, node, "slow effective timeout");
            state.slow = true;
          }
        }
        states.set(fn, state);
      }
    },
  };
});
