"use strict";

const { resolveModuleBinding } = require("./postgres-module-binding");

function isCursorModule(name, config) {
  return typeof name === "string" && config.modules.has(name);
}

function isCursorExecutor(name, config) {
  return typeof name === "string" && config.executors.has(name);
}

function namedCursorImport(context, identifier, helpers, config) {
  if (identifier?.type !== "Identifier") return null;
  const binding = resolveModuleBinding(identifier, context);
  if (binding?.kind !== "named" || !isCursorExecutor(binding.name, config)) return null;
  if (binding.status === "unresolved") {
    return !binding.source || isCursorModule(binding.source, config) ? "unresolved" : null;
  }
  return isCursorModule(binding.source, config) ? String(binding.name) : null;
}

function namespaceCursorStatus(context, node, config) {
  const binding = resolveModuleBinding(node, context);
  if (binding?.kind !== "namespace") return false;
  if (binding.status === "unresolved") {
    return !binding.source || isCursorModule(binding.source, config) ? "unresolved" : false;
  }
  return isCursorModule(binding.source, config);
}

function namespaceCursorImport(context, identifier, helpers, config) {
  return namespaceCursorStatus(context, identifier, config) === true;
}

function namespaceImportMember(context, node, helpers, config) {
  const member = helpers.unwrap(node);
  if (member?.type !== "MemberExpression") return null;
  const object = helpers.unwrap(member.object);
  if (object?.type !== "Identifier") return null;
  return namespaceCursorImport(context, object, helpers, config) === true ? member : null;
}

function namespaceCursorMember(context, node, helpers, config) {
  const member = helpers.unwrap(node);
  if (member?.type !== "MemberExpression") return null;
  const name = helpers.propertyName(member);
  if (!isCursorExecutor(name, config)) return null;
  const status = namespaceCursorStatus(context, helpers.unwrap(member.object), config);
  if (status === "unresolved") return "unresolved";
  return status === true ? String(name) : null;
}

function trustedTagName(source, name, config) {
  return (config.trustedSqlTags ?? []).some(
    (tag) =>
      tag.name === name &&
      (source == null || source === tag.module || String(source).startsWith(`${tag.module}/`)),
  );
}

function sqlTagBindingStatus(context, tag, helpers, config) {
  const value = helpers.unwrap(tag);
  if (value?.type === "MemberExpression") {
    const name = helpers.propertyName(value);
    const binding = resolveModuleBinding(helpers.unwrap(value.object), context);
    if (binding?.kind !== "namespace" || !name) return null;
    if (binding.status === "unresolved") {
      return trustedTagName(binding.source, name, config) ? "unresolved" : null;
    }
    return trustedTagName(binding.source, name, config) ? "trusted" : null;
  }
  if (value?.type !== "Identifier") return null;
  const binding = resolveModuleBinding(value, context);
  if (binding?.kind !== "named") return null;
  if (binding.status === "unresolved") {
    return trustedTagName(binding.source, binding.name, config) ? "unresolved" : null;
  }
  return trustedTagName(binding.source, binding.name, config) ? "trusted" : null;
}

module.exports = {
  isCursorExecutor,
  isCursorModule,
  namedCursorImport,
  namespaceCursorImport,
  namespaceCursorMember,
  namespaceImportMember,
  sqlTagBindingStatus,
};
