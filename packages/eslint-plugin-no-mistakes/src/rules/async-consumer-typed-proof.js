"use strict";
const { unwrapExpression } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { propertiesWritten } = require("./async-disposition-mocks");

function importedAwaited(call, index, context, sourceFacts) {
  const services = context.sourceCode.parserServices;
  // The dispatcher already validates the parser program and noResolve option.
  if (!services.esTreeNodeToTSNodeMap) return false;
  const callee = unwrapExpression(call.callee);
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
  for (const parameter of fn.parameters) visit(parameter);
  visit(fn.body);
  if (!safe || !count) return false;
  // An ES import cannot replace its exporter's binding. Only inspect that
  // implementation module, once per lint context, for local writes or escapes.
  const source = owner.getSourceFile();
  if (!sourceFacts.has(source)) {
    const unsafe = new Set();
    function inspect(current) {
      if (typeof current.escapedText === "string") {
        const referenced = checker.getSymbolAtLocation(current);
        const parent = current.parent;
        if (
          referenced &&
          referenced.valueDeclaration?.name !== current &&
          (parent.expression !== current || !Array.isArray(parent.arguments))
        )
          unsafe.add(referenced);
      }
      current.forEachChild(inspect);
    }
    inspect(source);
    sourceFacts.set(source, unsafe);
  }
  return !sourceFacts.get(source).has(symbol);
}

module.exports = { importedAwaited };
