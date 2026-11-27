"use strict";

const { staticImportSource } = require("./async-target-bindings");

function importedBindingName(node) {
  if (node.type === "ImportDefaultSpecifier") return "default";
  if (node.type === "ImportNamespaceSpecifier") return "*";
  return node.imported?.name ?? node.imported?.value ?? null;
}

function importRegistrar(def, modules, namespaceMember) {
  const source = staticImportSource(def);
  if (!source) return null;
  if (def.node.type === "TSImportEqualsDeclaration") {
    return modules.has(source) ? { kind: "untraced" } : null;
  }
  const imported = importedBindingName(def.node);
  if (!modules.has(source)) return imported === "test" ? { kind: "untraced" } : null;
  if (def.node.type === "ImportNamespaceSpecifier") return { kind: "namespace", source };
  return namespaceMember(source, imported);
}

function untracedCall(callNode, slot) {
  if (slot.kind === "fn") return { kind: "untraced", node: callNode, fn: slot.fn };
  return { kind: "opaque", node: callNode, reason: "an untraceable test binding" };
}

module.exports = { importRegistrar, untracedCall };
