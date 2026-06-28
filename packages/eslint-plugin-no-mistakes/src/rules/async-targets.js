"use strict";

const { unwrapExpression } = require("./async-ast");
const { propertyName } = require("./module-mock-helpers");

function safeRegExp(source) {
  try {
    return new RegExp(source);
  } catch {
    return null;
  }
}

function patternToRegExp(pattern) {
  if (pattern.startsWith("/") && pattern.endsWith("/") && pattern.length > 2) {
    return safeRegExp(pattern.slice(1, -1));
  }
  let source = "^";
  for (let index = 0; index < pattern.length; index += 1) {
    const ch = pattern[index];
    const next = pattern[index + 1];
    if (ch === "*" && next === "*") {
      if (pattern[index + 2] === "/") {
        source += "(?:.*/)?";
        index += 2;
      } else {
        source += ".*";
        index += 1;
      }
    } else if (ch === "*") {
      source += "[^/]*";
    } else if (ch === "?") {
      source += "[^/]";
    } else {
      source += ch.replace(/[\\^$+?.()|[\]{}]/g, "\\$&");
    }
  }
  return safeRegExp(`${source}$`);
}

function compileTargets(options, optionKey) {
  return (options[optionKey] || [])
    .map((target) => ({
      sourceSpecifierPatterns: (target.sourceSpecifierPatterns || [])
        .map(patternToRegExp)
        .filter(Boolean),
      calleeNamePatterns: (target.calleeNamePatterns || []).map(patternToRegExp).filter(Boolean),
    }))
    .filter(
      (target) => target.sourceSpecifierPatterns.length > 0 && target.calleeNamePatterns.length > 0,
    );
}

function matchesAny(value, patterns) {
  return typeof value === "string" && patterns.some((pattern) => pattern.test(value));
}

function targetMatches(targets, source, calleeName) {
  return targets.some(
    (target) =>
      matchesAny(source, target.sourceSpecifierPatterns) &&
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
  const expression = unwrapExpression(node);
  return expression?.type === "CallExpression" &&
    expression.callee.type === "Identifier" &&
    expression.callee.name === "require" &&
    typeof expression.arguments[0]?.value === "string"
    ? expression.arguments[0].value
    : null;
}

function bindingIdentifier(node) {
  if (node?.type === "Identifier") return node;
  return node?.type === "AssignmentPattern" && node.left.type === "Identifier" ? node.left : null;
}

function memberPropertyName(node) {
  if (!node.computed) return propertyName(node.property);
  return node.property?.type === "Literal" ? String(node.property.value) : null;
}

function createTargetMatcher(context, optionKey = "targets") {
  const targets = compileTargets(context.options?.[0] || {}, optionKey);
  const sourceSpecifierPatterns = targets.flatMap((target) => target.sourceSpecifierPatterns);
  const directBindings = new Map();
  const namespaceBindings = new Map();

  function recordDirect(id, source, calleeName) {
    if (id?.type !== "Identifier" || !targetMatches(targets, source, calleeName)) return;
    const variable = resolveVariable(id, context);
    if (variable) directBindings.set(variable, { source, calleeName });
  }

  function recordNamespace(id, source) {
    if (id.type !== "Identifier" || !matchesAny(source, sourceSpecifierPatterns)) {
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
        recordDirect(bindingIdentifier(property.value), source, propertyName(property.key));
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
    const source =
      requireSource(node.object) ||
      (node.object.type === "Identifier"
        ? namespaceBindings.get(resolveVariable(node.object, context))
        : null);
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
