"use strict";

const { posix } = require("node:path");
const { unwrapExpression } = require("./async-ast");
const { memberPropertyName, resolveVariable } = require("./async-target-bindings");
const { literalString } = require("./logical-call");
const { isUnshadowedGlobal } = require("./test-no-fixed-temp-paths-bindings");

const DEFAULT_FIXTURE_DIRS = ["fixtures", "__fixtures__", "test-cases"];
const MKDTEMP_METHODS = new Set([
  "fs.mkdtempSync",
  "fs.mkdtempDisposableSync",
  "fs.promises.mkdtemp",
  "fs.promises.mkdtempDisposable",
  "fs/promises.mkdtemp",
  "fs/promises.mkdtempDisposable",
]);
const PATH_JOIN_METHODS = new Set(["join", "resolve", "normalize"]);
const TEMP_PREFIXES = /^\/(?:var\/)?tmp(?:\/|$)/;

function normalizeFixtureDirs(configured) {
  const raw = configured ?? DEFAULT_FIXTURE_DIRS;
  return new Set(raw.map((entry) => entry.replace(/^\/+|\/+$/g, "")).filter(Boolean));
}

function pathVariable(node, context) {
  const expr = unwrapExpression(node);
  return expr?.type === "Identifier" ? resolveVariable(expr, context) : null;
}

function isImportMetaProp(node, prop) {
  return (
    node?.type === "MemberExpression" &&
    node.object?.type === "MetaProperty" &&
    memberPropertyName(node) === prop
  );
}

function normalizePathResult(anchor, rawSegments, fixtureDirs) {
  const joined = rawSegments.join("/").replace(/\\/g, "/");
  if (anchor === "cwd" && TEMP_PREFIXES.test(joined)) return { kind: "temp" };
  const normalized = posix.normalize(joined);
  const segments = normalized.split("/").filter((s) => s && s !== ".");
  if (segments.some((segment) => fixtureDirs.has(segment))) return { kind: "fixture" };
  return { kind: "repo", anchor, key: `${anchor}:${normalized}` };
}

function isTempAnchor(node, resolve) {
  const expr = unwrapExpression(node);
  if (expr?.type === "AwaitExpression") return isTempAnchor(expr.argument, resolve);
  if (expr?.type === "MemberExpression" && memberPropertyName(expr) === "path") {
    return isTempAnchor(expr.object, resolve);
  }
  if (expr?.type !== "CallExpression") return false;
  const binding = resolve(expr.callee);
  return binding === "os.tmpdir" || MKDTEMP_METHODS.has(binding);
}

function constInitOf(node, context, seen) {
  const variable = resolveVariable(node, context);
  if (!variable || seen.has(variable) || variable.defs.length !== 1) return null;
  const def = variable.defs[0];
  if (def.type !== "Variable" || def.parent.kind !== "const" || def.node.id.type !== "Identifier") {
    return null;
  }
  seen.add(variable);
  return def.node.init;
}

function createPathResolver(context, resolve, fixtureDirs) {
  function resolveStringArg(input, env, seen) {
    const node = unwrapExpression(input);
    const direct = literalString(node);
    if (direct !== null) return direct;
    if (node?.type !== "Identifier") return null;
    if (env?.has(node.name)) return resolveStringArg(env.get(node.name), null, seen);
    const init = constInitOf(node, context, seen);
    return init ? resolveStringArg(init, env, seen) : null;
  }

  function resolvePath(input, env = null, seen = new Set()) {
    const node = unwrapExpression(input);
    if (!node) return null;
    if (isTempAnchor(node, resolve)) return { kind: "temp" };
    if (node.type === "Identifier") {
      if (env?.has(node.name)) return resolvePath(env.get(node.name), null, seen);
      if (isUnshadowedGlobal(node, "__dirname", context)) {
        return { kind: "anchor", anchor: "fileDir", raw: [] };
      }
      const init = constInitOf(node, context, seen);
      return init ? resolvePath(init, env, seen) : null;
    }
    const lit = literalString(node);
    if (lit !== null) return normalizePathResult("cwd", [lit], fixtureDirs);
    if (isImportMetaProp(node, "dirname")) return { kind: "anchor", anchor: "fileDir", raw: [] };
    if (
      node.type === "NewExpression" &&
      isUnshadowedGlobal(unwrapExpression(node.callee), "URL", context)
    ) {
      const rel = literalString(node.arguments[0]);
      if (rel !== null && isImportMetaProp(unwrapExpression(node.arguments[1]), "url")) {
        return normalizePathResult("fileDir", [rel], fixtureDirs);
      }
    }
    return node.type === "CallExpression" ? resolveCallPath(node, env, seen) : null;
  }

  function resolveCallPath(node, env, seen) {
    const callee = unwrapExpression(node.callee);
    const binding = resolve(callee);
    if (
      callee?.type === "MemberExpression" &&
      memberPropertyName(callee) === "cwd" &&
      isUnshadowedGlobal(unwrapExpression(callee.object), "process", context)
    ) {
      return { kind: "anchor", anchor: "cwd", raw: [] };
    }
    if (binding === "url.fileURLToPath") return resolvePath(node.arguments[0], env, seen);
    if (binding === "path.dirname") return resolveDirnamePath(node.arguments[0], env, seen);
    if (binding?.startsWith("path.") && PATH_JOIN_METHODS.has(binding.slice(5))) {
      return resolveJoinedPath(node.arguments, env, seen);
    }
    return null;
  }

  function resolveDirnamePath(arg, env, seen) {
    const inner = unwrapExpression(arg);
    if (
      inner?.type === "CallExpression" &&
      resolve(inner.callee) === "url.fileURLToPath" &&
      isImportMetaProp(unwrapExpression(inner.arguments[0]), "url")
    ) {
      return { kind: "anchor", anchor: "fileDir", raw: [] };
    }
    const target = resolvePath(inner, env, seen);
    if (target?.kind === "temp") return { kind: "temp" };
    if (target?.kind !== "repo") return null;
    return normalizePathResult(
      target.anchor,
      [target.key.slice(target.anchor.length + 1), ".."],
      fixtureDirs,
    );
  }

  function resolveJoinedPath(args, env, seen) {
    let anchor = "cwd";
    const parts = [];
    for (let i = 0; i < args.length; i += 1) {
      if (args[i].type === "SpreadElement") return null;
      const lit = resolveStringArg(args[i], env, new Set(seen));
      if (lit !== null) {
        parts.push(lit);
        continue;
      }
      const resolved = resolvePath(args[i], env, new Set(seen));
      if (!resolved) return null;
      if (resolved.kind === "temp" || resolved.kind === "fixture") return resolved;
      if (i === 0) {
        anchor = resolved.anchor;
        parts.push(
          ...(resolved.kind === "anchor"
            ? resolved.raw
            : [resolved.key.slice(resolved.anchor.length + 1)]),
        );
      } else {
        parts.push(resolved.key.slice(resolved.anchor.length + 1));
      }
    }
    return normalizePathResult(anchor, parts, fixtureDirs);
  }

  return resolvePath;
}

module.exports = { createPathResolver, normalizeFixtureDirs, pathVariable };
