"use strict";

const { propertyName } = require("./module-mock-helpers");

function safeRegExp(source) {
  try {
    return new RegExp(source);
  } catch {
    return null;
  }
}

function compileTargets(options) {
  return (options.targets || [])
    .map((target) => ({
      sourcePatterns: (target.sourcePatterns || []).map(safeRegExp).filter(Boolean),
      calleeNamePatterns: (target.calleeNamePatterns || []).map(safeRegExp).filter(Boolean),
    }))
    .filter((target) => target.sourcePatterns.length > 0 && target.calleeNamePatterns.length > 0);
}

function matchesAny(value, patterns) {
  return typeof value === "string" && patterns.some((pattern) => pattern.test(value));
}

function targetMatches(targets, source, calleeName) {
  return targets.some(
    (target) =>
      matchesAny(source, target.sourcePatterns) &&
      matchesAny(calleeName, target.calleeNamePatterns),
  );
}

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

function requireSource(node) {
  return node?.type === "CallExpression" &&
    node.callee.type === "Identifier" &&
    node.callee.name === "require" &&
    typeof node.arguments[0]?.value === "string"
    ? node.arguments[0].value
    : null;
}

function memberPropertyName(node) {
  if (!node.computed) return propertyName(node.property);
  return node.property?.type === "Literal" ? String(node.property.value) : null;
}

function createTargetMatcher(context) {
  const targets = compileTargets(context.options?.[0] || {});
  const sourcePatterns = targets.flatMap((target) => target.sourcePatterns);
  const directBindings = new Map();
  const namespaceBindings = new Map();

  function recordDirect(id, source, calleeName) {
    if (id?.type !== "Identifier" || !targetMatches(targets, source, calleeName)) return;
    const variable = resolveVariable(id, context);
    if (variable) directBindings.set(variable, { source, calleeName });
  }

  function recordNamespace(id, source) {
    if (id.type !== "Identifier" || !matchesAny(source, sourcePatterns)) {
      return;
    }
    const variable = resolveVariable(id, context);
    if (variable) namespaceBindings.set(variable, source);
  }

  function recordRequireDeclarator(node) {
    const source = requireSource(node.init);
    if (!source) return;
    if (node.id.type === "Identifier") {
      recordNamespace(node.id, source);
      recordDirect(node.id, source, node.id.name);
      return;
    }
    if (node.id.type === "ObjectPattern") {
      for (const property of node.id.properties) {
        if (property.type !== "Property") continue;
        recordDirect(property.value, source, propertyName(property.key));
      }
    }
  }

  function recordProgramRequires(node) {
    for (const statement of node.body) {
      const declarations =
        statement.type === "VariableDeclaration"
          ? statement.declarations
          : statement.type === "ExportNamedDeclaration" &&
              statement.declaration?.type === "VariableDeclaration"
            ? statement.declaration.declarations
            : [];
      for (const declaration of declarations) recordRequireDeclarator(declaration);
    }
  }

  function isDirectTarget(node) {
    if (node.type !== "Identifier") return false;
    const variable = resolveVariable(node, context);
    return Boolean(variable && directBindings.has(variable));
  }

  function isNamespaceTarget(node) {
    if (node.type !== "MemberExpression") return false;
    const name = memberPropertyName(node);
    if (!name) return false;
    if (node.object.type !== "Identifier") return false;
    const variable = resolveVariable(node.object, context);
    const source = variable ? namespaceBindings.get(variable) : null;
    return Boolean(source && targetMatches(targets, source, name));
  }

  return {
    hasTargets: targets.length > 0,
    isTargetCall(node) {
      return isDirectTarget(node.callee) || isNamespaceTarget(node.callee);
    },
    visitors: {
      Program: recordProgramRequires,
      ImportDeclaration(node) {
        const source = node.source.value;
        for (const specifier of node.specifiers) {
          if (specifier.type === "ImportNamespaceSpecifier") {
            recordNamespace(specifier.local, source);
          } else if (specifier.type === "ImportDefaultSpecifier") {
            recordDirect(specifier.local, source, specifier.local.name);
          } else if (specifier.type === "ImportSpecifier") {
            recordDirect(specifier.local, source, importSpecifierName(specifier));
          }
        }
      },
      VariableDeclarator(node) {
        recordRequireDeclarator(node);
      },
    },
  };
}

module.exports = {
  createTargetMatcher,
  memberPropertyName,
};
