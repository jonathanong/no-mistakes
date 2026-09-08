"use strict";

const { unwrapExpression } = require("./async-ast");
const { compileTargets, matchesAny, targetMatches } = require("./async-patterns");
const { propertyName, literalString } = require("./module-mock-helpers");

function findVariable(scope, name) {
  while (scope) {
    const variable = scope.variables.find((candidate) => candidate.name === name);
    if (variable) return variable;
    scope = scope.upper;
  }
  return null;
}

function resolveVariable(node, context) {
  return findVariable(context.sourceCode.getScope(node), node.name);
}

function importSpecifierName(specifier) {
  const imported = specifier.imported;
  return imported.type === "Literal" ? String(imported.value) : imported.name;
}

function isLocalRequire(id, context) {
  const variable = resolveVariable(id, context);
  return Boolean(variable?.defs.some((def) => def.type && def.type !== "ImplicitGlobalVariable"));
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

function bindingIdentifier(node) {
  if (node?.type === "Identifier") return node;
  return node?.type === "AssignmentPattern" && node.left.type === "Identifier" ? node.left : null;
}

function memberPropertyName(node) {
  if (!node.computed) return propertyName(node.property);
  return staticComputedPropertyName(node.property);
}

function staticComputedPropertyName(node) {
  return literalString(unwrapExpression(node));
}

function createTargetMatcher(context, optionKey = "targets") {
  const targets = compileTargets(context.options?.[0] || {}, optionKey);
  const sourceSpecifierPatterns = targets.flatMap((target) => target.sourceSpecifierPatterns);
  const directBindings = new Map();
  const namespaceBindings = new Map();

  function isReassigned(id) {
    const variable = resolveVariable(id, context);
    const writes = variable?.references.filter((reference) => reference.isWrite()) || [];
    const initializationSites = new Set(
      writes.filter((reference) => reference.init).map((reference) => reference.identifier),
    );
    const isVar = variable?.defs.some(
      (definition) => definition.type === "Variable" && definition.parent?.kind === "var",
    );
    return writes.some((reference) => !reference.init) || (isVar && initializationSites.size > 1);
  }

  function recordDirect(id, source, calleeName) {
    if (
      id?.type !== "Identifier" ||
      isReassigned(id) ||
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
      isReassigned(id) ||
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
        for (const property of node.id.properties) {
          if (property.type !== "Property") continue;
          const name = property.computed
            ? staticComputedPropertyName(property.key)
            : propertyName(property.key);
          if (name) recordDirect(bindingIdentifier(property.value), source, name);
        }
      }
      return;
    }
    const member = unwrapExpression(node.init);
    if (member?.type !== "MemberExpression" || node.id.type !== "Identifier") return;
    const memberSource = requireSource(member.object, context);
    const name = memberPropertyName(member);
    if (memberSource && name) recordDirect(node.id, memberSource, name);
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
      else if (child.type === "VariableDeclarator") recordRequireDeclarator(child);
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

  function resolveCallTarget(node) {
    const callee = unwrapExpression(node.callee);
    return resolveDirectTarget(callee) || resolveNamespaceTarget(callee);
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
