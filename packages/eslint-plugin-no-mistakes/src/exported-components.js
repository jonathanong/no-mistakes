"use strict";

const { functionFromExpression, nonEmptyStrings } = require("./component-functions");

const DEFAULT_COMPONENT_NAME_PATTERN = "^[A-Z]";
const DEFAULT_WRAPPERS = ["memo", "forwardRef", "observer"];
const DEFAULT_EXPORT_TYPES = ["named", "default"];

function normalizedComponentOptions(option) {
  return {
    componentNamePattern: compilePattern(
      option.componentNamePattern,
      DEFAULT_COMPONENT_NAME_PATTERN,
    ),
    components: compileMatchers(option.components),
    ignoreComponents: compileMatchers(option.ignoreComponents),
    wrappers: new Set(nonEmptyStrings(option.wrappers, DEFAULT_WRAPPERS)),
    exportTypes: new Set(nonEmptyStrings(option.exportTypes, DEFAULT_EXPORT_TYPES)),
    checkAnonymousDefault: option.checkAnonymousDefault === true,
  };
}

function compilePattern(value, fallback) {
  return new RegExp(typeof value === "string" && value.length > 0 ? value : fallback);
}

function compileMatchers(values) {
  if (!Array.isArray(values)) {
    return [];
  }
  return values
    .filter((value) => typeof value === "string" && value.length > 0)
    .map((value) => {
      const regex = regexLiteral(value);
      return regex ? { regex } : { exact: value };
    });
}

function regexLiteral(value) {
  if (!value.startsWith("/") || value.lastIndexOf("/") === 0) {
    return null;
  }
  const lastSlash = value.lastIndexOf("/");
  return new RegExp(value.slice(1, lastSlash), value.slice(lastSlash + 1));
}

function shouldCheckComponent(component, opts) {
  if (component.anonymousDefault) {
    return opts.checkAnonymousDefault;
  }
  if (opts.ignoreComponents.some((matcher) => matchesName(component.name, matcher))) {
    return false;
  }
  if (opts.components.length > 0) {
    return opts.components.some((matcher) => matchesName(component.name, matcher));
  }
  return opts.componentNamePattern.test(component.name);
}

function matchesName(name, matcher) {
  return matcher.regex ? matcher.regex.test(name) : matcher.exact === name;
}

function collectExportedComponents(program, opts) {
  const definitions = new Map();
  const namedExports = new Set();
  const defaultExports = new Set();
  const components = [];

  for (const statement of program.body) {
    collectTopLevelDefinition(statement, definitions, opts);
    if (statement.type === "ExportNamedDeclaration") {
      collectNamedExport(statement, definitions, namedExports, defaultExports, components, opts);
    }
    if (statement.type === "ExportDefaultDeclaration") {
      collectDefaultExport(statement, definitions, defaultExports, components, opts);
    }
  }

  pushExportedDefinitions(namedExports, definitions, components, opts.exportTypes.has("named"));
  pushExportedDefinitions(defaultExports, definitions, components, opts.exportTypes.has("default"));
  return uniqueComponents(components);
}

function pushExportedDefinitions(names, definitions, components, enabled) {
  if (!enabled) {
    return;
  }
  for (const name of names) {
    const definition = definitions.get(name);
    if (definition) {
      components.push(definition);
    }
  }
}

function collectTopLevelDefinition(statement, definitions, opts) {
  if (statement.type === "FunctionDeclaration" && statement.id) {
    definitions.set(statement.id.name, { name: statement.id.name, fn: statement });
    return;
  }
  if (statement.type !== "VariableDeclaration") {
    return;
  }
  for (const declaration of statement.declarations) {
    if (declaration.id.type !== "Identifier") {
      continue;
    }
    const fn = functionFromExpression(declaration.init, opts);
    if (fn) {
      definitions.set(declaration.id.name, { name: declaration.id.name, fn });
    }
  }
}

function collectNamedExport(
  statement,
  definitions,
  namedExports,
  defaultExports,
  components,
  opts,
) {
  if (statement.source) {
    return;
  }
  if (statement.declaration?.type === "FunctionDeclaration" && statement.declaration.id) {
    const name = statement.declaration.id.name;
    definitions.set(name, { name, fn: statement.declaration });
    if (opts.exportTypes.has("named")) {
      components.push(definitions.get(name));
    }
    return;
  }
  if (statement.declaration?.type === "VariableDeclaration") {
    collectTopLevelDefinition(statement.declaration, definitions, opts);
    for (const declaration of statement.declaration.declarations) {
      const definition = definitions.get(declaration.id.name);
      if (declaration.id.type === "Identifier" && definition && opts.exportTypes.has("named")) {
        components.push(definition);
      }
    }
    return;
  }
  for (const specifier of statement.specifiers || []) {
    if (specifier.local?.type === "Identifier") {
      if (specifier.exported?.name === "default") {
        defaultExports.add(specifier.local.name);
      } else {
        namedExports.add(specifier.local.name);
      }
    }
  }
}

function collectDefaultExport(statement, definitions, defaultExports, components, opts) {
  if (!opts.exportTypes.has("default")) {
    return;
  }
  const declaration = statement.declaration;
  if (declaration.type === "Identifier") {
    defaultExports.add(declaration.name);
    return;
  }
  if (declaration.type === "FunctionDeclaration") {
    const name = declaration.id?.name || "default";
    if (declaration.id || opts.checkAnonymousDefault) {
      components.push({ name, fn: declaration, anonymousDefault: !declaration.id });
    }
    return;
  }
  const fn = functionFromExpression(declaration, opts);
  if (fn && opts.checkAnonymousDefault) {
    components.push({ name: fn.id?.name || "default", fn, anonymousDefault: !fn.id });
  }
  if (fn?.id) {
    definitions.set(fn.id.name, { name: fn.id.name, fn });
    defaultExports.add(fn.id.name);
  }
}

function uniqueComponents(components) {
  const seen = new Set();
  return components.filter((component) => {
    const key = `${component.name}:${component.fn.range?.[0] ?? component.fn.loc?.start?.line ?? ""}`;
    if (seen.has(key)) {
      return false;
    }
    seen.add(key);
    return true;
  });
}

module.exports = {
  collectExportedComponents,
  normalizedComponentOptions,
  shouldCheckComponent,
};
