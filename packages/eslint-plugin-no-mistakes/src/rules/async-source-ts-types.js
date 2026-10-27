"use strict";

// Preserve erased syntax and its references without inferring these types.
function compositeType(node, { ts, convert, make, list }) {
  if (ts.isImportTypeNode(node))
    return make(node, "TSImportType", {
      argument: convert(node.argument),
      qualifier: convert(node.qualifier),
      typeArguments:
        node.typeArguments &&
        make(node, "TSTypeParameterInstantiation", { params: list(node.typeArguments) }),
      options: convert(node.attributes),
      isTypeOf: node.isTypeOf,
    });
  if (ts.isTupleTypeNode(node))
    return make(node, "TSTupleType", { elementTypes: list(node.elements) });
  if (ts.isIndexedAccessTypeNode(node))
    return make(node, "TSIndexedAccessType", {
      objectType: convert(node.objectType),
      indexType: convert(node.indexType),
    });
  if (ts.isConditionalTypeNode(node))
    return make(node, "TSConditionalType", {
      checkType: convert(node.checkType),
      extendsType: convert(node.extendsType),
      trueType: convert(node.trueType),
      falseType: convert(node.falseType),
    });
  if (ts.isInferTypeNode(node))
    return make(node, "TSInferType", { typeParameter: convert(node.typeParameter) });
  if (ts.isMappedTypeNode(node)) {
    const modifier = (token) =>
      !token
        ? undefined
        : [ts.SyntaxKind.PlusToken, ts.SyntaxKind.MinusToken].includes(token.kind)
          ? ts.tokenToString(token.kind)
          : true;
    return make(node, "TSMappedType", {
      typeParameter: convert(node.typeParameter),
      nameType: convert(node.nameType),
      typeAnnotation: convert(node.type),
      readonly: modifier(node.readonlyToken),
      optional: modifier(node.questionToken),
    });
  }
  return null;
}

module.exports = { compositeType };
