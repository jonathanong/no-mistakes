"use strict";
const { unwrapExpression } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { propertiesWritten } = require("./async-disposition-mocks");

function importedAwaited(call, index, context) {
  const services = context.sourceCode.parserServices;
  if (
    !services?.program ||
    services.program.getCompilerOptions().noResolve ||
    !services.esTreeNodeToTSNodeMap
  )
    return false;
  const callee = unwrapExpression(call.callee);
  if (callee?.type !== "Identifier") return false;
  const binding = resolveVariable(callee, context);
  if (
    binding?.defs.length !== 1 ||
    binding.defs[0].type !== "ImportBinding" ||
    binding.references.some((ref) => ref.isWrite()) ||
    propertiesWritten(callee, context, new Set(), null, true)
  )
    return false;
  const checker = services.program.getTypeChecker();
  const node = services.esTreeNodeToTSNodeMap.get(callee);
  const imported = node && checker.getSymbolAtLocation(node);
  if (!imported) return false;
  // The ESLint binding above is an import, so its checker symbol is an alias.
  const symbol = checker.getAliasedSymbol(imported);
  if (symbol?.declarations?.length !== 1) return false;
  const owner = symbol.declarations[0];
  const fn = owner.body ? owner : owner.initializer;
  if (
    !fn?.body ||
    (owner.initializer && owner.parent.getFirstToken().getText() !== "const") ||
    fn.parameters.some((param) => param.dotDotDotToken)
  )
    return false;
  const parameter = fn.parameters[index];
  if (!parameter || typeof parameter.name.escapedText !== "string") return false;
  const target = checker.getSymbolAtLocation(parameter.name);
  let count = 0;
  let safe = true;
  // Reuse the parser-owned program, including symbol identity for shadowed names.
  function visit(current) {
    if (!safe) return;
    if (
      current.escapedText === "arguments" ||
      (current.escapedText === "eval" &&
        current.parent.expression === current &&
        Array.isArray(current.parent.arguments))
    )
      safe = false;
    if (
      current !== parameter.name &&
      typeof current.escapedText === "string" &&
      checker.getSymbolAtLocation(current) === target
    ) {
      count++;
      for (let owner = current.parent; owner && owner !== fn; owner = owner.parent)
        if (owner.parameters && owner.body) safe = false;
      const invocation = current.parent;
      if (
        !Array.isArray(invocation.arguments) ||
        invocation.expression !== current ||
        invocation.questionDotToken ||
        invocation.parent?.expression !== invocation ||
        invocation.parent.getFirstToken().getText() !== "await"
      )
        safe = false;
    }
    current.forEachChild(visit);
  }
  visit(fn.body);
  if (!safe || !count) return false;
  // Reject writes or escaping uses of the implementation anywhere in the borrowed program.
  for (const source of services.program.getSourceFiles()) {
    if (source.isDeclarationFile) continue;
    function inspect(current) {
      if (!safe) return;
      if (
        typeof current.escapedText === "string" &&
        current !== fn.name &&
        current !== owner.name
      ) {
        let referenced = checker.getSymbolAtLocation(current);
        // TypeScript SymbolFlags.Alias; resolve only aliases supplied by the checker.
        if (referenced?.flags & 2097152) referenced = checker.getAliasedSymbol(referenced);
        if (referenced === symbol) {
          const parent = current.parent;
          const importName =
            parent.parent?.parent?.parent?.moduleSpecifier ||
            parent.parent?.parent?.moduleSpecifier ||
            parent.parent?.moduleSpecifier;
          if (!importName && (parent.expression !== current || !Array.isArray(parent.arguments)))
            safe = false;
        }
      }
      current.forEachChild(inspect);
    }
    inspect(source);
  }
  return safe;
}

module.exports = { importedAwaited };
