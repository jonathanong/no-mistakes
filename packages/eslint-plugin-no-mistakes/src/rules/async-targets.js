"use strict";

const { unwrapExpression } = require("./async-ast");
const {
  isLocalRequire,
  isReassigned,
  memberPropertyName,
  namespaceSourceFromInit,
  recordObjectPatternBindings,
  resolveVariable,
} = require("./async-target-bindings");
const { compileTargets, matchesAny, targetMatches } = require("./async-patterns");
const { literalString, logicalCall } = require("./module-mock-helpers");

function importSpecifierName(specifier) {
  const imported = specifier.imported;
  return imported.type === "Literal" ? String(imported.value) : imported.name;
}

function requireSource(node, context) {
  const expression = unwrapExpression(node);
  const source = literalString(unwrapExpression(expression?.arguments?.[0]));
  if (
    expression?.type !== "CallExpression" ||
    expression.callee.type !== "Identifier" ||
    expression.callee.name !== "require" ||
    source === null ||
    isLocalRequire(expression.callee, context)
  ) {
    return null;
  }
  return source;
}

function createTargetMatcher(context, optionKey = "targets") {
  const targets = compileTargets(context.options?.[0] || {}, optionKey);
  const sourceSpecifierPatterns = targets.flatMap((target) => target.sourceSpecifierPatterns);
  const directBindings = new Map();
  const namespaceBindings = new Map();

  function recordDirect(id, source, calleeName) {
    if (
      id?.type !== "Identifier" ||
      isReassigned(id, context) ||
      !targetMatches(targets, source, calleeName)
    ) {
      return;
    }
    const variable = resolveVariable(id, context);
    if (variable) directBindings.set(variable, { source, calleeName });
  }

  function recordNamespace(id, source) {
    if (
      id.type !== "Identifier" ||
      isReassigned(id, context) ||
      !matchesAny(source, sourceSpecifierPatterns)
    ) {
      return;
    }
    const variable = resolveVariable(id, context);
    if (variable) namespaceBindings.set(variable, source);
  }

  function recordRequireDeclarator(node) {
    if (node.parent?.type !== "VariableDeclaration") return;
    const source = requireSource(node.init, context);
    if (source) {
      if (node.id.type === "Identifier") {
        recordNamespace(node.id, source);
        recordDirect(node.id, source, node.id.name);
        return;
      }
      if (node.id.type === "ObjectPattern") {
        recordObjectPatternBindings(node.id, source, recordDirect);
      }
      return;
    }
    const member = unwrapExpression(node.init);
    if (member?.type === "MemberExpression" && node.id.type === "Identifier") {
      const memberSource = requireSource(member.object, context);
      const name = memberPropertyName(member);
      if (memberSource && name) recordDirect(node.id, memberSource, name);
      return;
    }
    if (node.id.type !== "ObjectPattern") return;
    const nsSource = namespaceSourceFromInit(node.init, context, namespaceBindings);
    if (nsSource) recordObjectPatternBindings(node.id, nsSource, recordDirect);
  }

  function recordImportEquals(node) {
    if (node.importKind === "type" || node.id?.type !== "Identifier") return;
    const ref = node.moduleReference;
    if (ref?.type !== "TSExternalModuleReference") return;
    const source = ref.expression?.type === "Literal" ? String(ref.expression.value) : null;
    if (!source) return;
    recordNamespace(node.id, source);
    recordDirect(node.id, source, node.id.name);
  }

  function recordImportDeclaration(node) {
    const source = node.source.value;
    for (const specifier of node.specifiers) {
      if (specifier.type === "ImportNamespaceSpecifier") {
        recordNamespace(specifier.local, source);
      } else if (specifier.type === "ImportDefaultSpecifier") {
        recordNamespace(specifier.local, source);
        recordDirect(specifier.local, source, specifier.local.name);
      } else if (specifier.type === "ImportSpecifier") {
        const imported = importSpecifierName(specifier);
        recordDirect(
          specifier.local,
          source,
          imported === "default" ? specifier.local.name : imported,
        );
      }
    }
  }

  function walk(node, visit) {
    if (!node?.type) return;
    visit(node);
    for (const key of context.sourceCode.visitorKeys[node.type] || []) {
      const value = node[key];
      if (Array.isArray(value)) {
        for (const child of value) walk(child, visit);
      } else {
        walk(value, visit);
      }
    }
  }

  function recordProgram(node) {
    walk(node, (child) => {
      if (child.type === "ImportDeclaration") recordImportDeclaration(child);
      else if (child.type === "TSImportEqualsDeclaration") recordImportEquals(child);
    });
    walk(node, (child) => {
      if (child.type === "VariableDeclarator") recordRequireDeclarator(child);
    });
  }

  function resolveDirectTarget(node) {
    if (node.type !== "Identifier") return null;
    const variable = resolveVariable(node, context);
    return (variable && directBindings.get(variable)) || null;
  }

  function resolveNamespaceTarget(node) {
    if (node.type !== "MemberExpression") return null;
    const name = memberPropertyName(node);
    if (!name) return null;
    const object = unwrapExpression(node.object);
    const source =
      requireSource(object, context) ||
      (object.type === "Identifier"
        ? namespaceBindings.get(resolveVariable(object, context))
        : null);
    return source && targetMatches(targets, source, name) ? { source, calleeName: name } : null;
  }

  function resolveCallee(callee, seen = new Set()) {
    callee = unwrapExpression(callee);
    if (!callee) return null;
    const found = resolveDirectTarget(callee) || resolveNamespaceTarget(callee);
    if (found) return found;
    if (callee.type !== "Identifier") return null;
    const variable = resolveVariable(callee, context);
    if (
      !variable ||
      seen.has(variable) ||
      variable.defs.length !== 1 ||
      isReassigned(callee, context)
    ) {
      return null;
    }
    const def = variable.defs[0];
    if (
      def.type !== "Variable" ||
      def.parent?.kind !== "const" ||
      def.node?.id?.type !== "Identifier"
    ) {
      return null;
    }
    seen.add(variable);
    return resolveCallee(def.node.init, seen);
  }

  function resolveCallTarget(node) {
    if (node?.type !== "CallExpression") return null;
    const direct = resolveCallee(node.callee);
    if (direct) return { ...direct, arguments: node.arguments };
    const invoked = logicalCall(node);
    if (!invoked) return null;
    const forwarded = resolveCallee(invoked.callee);
    return forwarded ? { ...forwarded, arguments: invoked.arguments } : null;
  }

  return {
    hasTargets: targets.length > 0,
    resolveCallTarget,
    isTargetCall(node) {
      return Boolean(resolveCallTarget(node));
    },
    visitors: {
      Program: recordProgram,
      ImportDeclaration: recordImportDeclaration,
      TSImportEqualsDeclaration: recordImportEquals,
      VariableDeclarator: recordRequireDeclarator,
    },
  };
}

module.exports = {
  createTargetMatcher,
  memberPropertyName,
};
