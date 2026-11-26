"use strict";

const { keyName, reactTypeStatus, typeAnnotation, typeName } = require("../react-node-types");

function classifiedProps(type, facts) {
  if (type && type.type === "TSTypeLiteral") {
    const props = new Set();
    const unresolved = new Set();
    for (const member of type.members) {
      if (member.type !== "TSPropertySignature") continue;
      const name = keyName(member.key);
      if (!name) continue;
      const status = reactTypeStatus(typeAnnotation(member), facts);
      if (status === "yes") props.add(name);
      else if (status === "unresolved") unresolved.add(name);
    }
    return { props, unresolved };
  }
  const name = typeName(type);
  return {
    props: name ? facts.objectProps.get(name) || null : null,
    unresolved: null,
  };
}

module.exports = { classifiedProps };
