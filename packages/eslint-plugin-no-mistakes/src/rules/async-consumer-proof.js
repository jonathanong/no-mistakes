"use strict";

const { unwrapExpression, unwrapTransparentParent } = require("./async-ast");
const { resolveVariable, stableConstInitializer } = require("./async-target-bindings");
const { propertiesWritten } = require("./async-disposition-mocks");

function localFunction(id, context, seen = new Set()) {
  if (id?.type !== "Identifier") return null;
  const variable = resolveVariable(id, context);
  if (
    !variable ||
    seen.has(variable) ||
    variable.defs.length !== 1 ||
    variable.references.some((ref) => ref.isWrite() && !ref.init) ||
    propertiesWritten(id, context, new Set(), null, true)
  )
    return null;
  const def = variable.defs[0];
  if (def.type === "FunctionName" && def.node.type === "FunctionDeclaration") return def.node;
  const init = unwrapExpression(stableConstInitializer(id, context, seen));
  return ["ArrowFunctionExpression", "FunctionExpression"].includes(init?.type)
    ? init
    : localFunction(init, context, seen);
}

function locallyAwaited(call, index, context) {
  const fn = localFunction(unwrapExpression(call.callee), context);
  const param = fn?.params[index];
  if (!param || param.type !== "Identifier" || fn.params.some((p) => p.type === "RestElement"))
    return false;
  const variable = resolveVariable(param, context);
  const reads = variable?.references.filter((ref) => !ref.init) || [];
  return (
    reads.length > 0 &&
    reads.every((ref) => {
      if (ref.isWrite()) return false;
      const value = unwrapTransparentParent(ref.identifier);
      const invocation = value.parent;
      if (
        invocation?.type !== "CallExpression" ||
        invocation.callee !== value ||
        invocation.optional
      )
        return false;
      const result = unwrapTransparentParent(invocation);
      return result.parent?.type === "AwaitExpression" && result.parent.argument === result;
    })
  );
}

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
  let symbol;
  try {
    symbol = checker.getAliasedSymbol(imported);
  } catch {
    return false;
  }
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
      current !== parameter.name &&
      typeof current.escapedText === "string" &&
      checker.getSymbolAtLocation(current) === target
    ) {
      count++;
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
            parent.parent?.parent?.moduleSpecifier || parent.parent?.moduleSpecifier;
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

function provenAwaitingConsumer(argument, context) {
  const value = unwrapTransparentParent(argument);
  const call = value.parent;
  if (
    call?.type !== "CallExpression" ||
    call.optional ||
    call.arguments.some((arg) => arg.type === "SpreadElement")
  )
    return false;
  const index = call.arguments.indexOf(value);
  return (
    index >= 0 && (locallyAwaited(call, index, context) || importedAwaited(call, index, context))
  );
}

module.exports = { provenAwaitingConsumer };
