// Each nested arm shadows independently; reads outside it must survive.
// Static [0] writes intentionally exercise identity-only receiver dependencies.
export function regular(parameter) {
  if (condition) {
    let branchLocal = outside;
    if (innerCondition) { let outside = parameter; consume(outside, this, arguments); }
    else { consume(branchLocal, outside); }
  } else { var alternate; consume(alternate, outside); }
  outside.value = parameter;
  identityOnly[0] = parameter;
  return outside;
}
export const arrow = (parameter) => {
  if (condition) { let local = outside; consume(local, this, arguments); }
  else { const outside = parameter; consume(outside); }
  identityOnly[0] = parameter;
  return outside;
};
export const named = function recursive(parameter) {
  if (condition) { function local() { return outside; } consume(local, recursive); }
  else { let recursive = outside; consume(recursive); }
  identityOnly[0] = parameter;
  return outside;
};

// These are distinct supported expression projections, not executed fixtures.
export const expressions = () => {
  const dynamicObject = { [objectKey]: objectValue, ...objectSpread };
  const optional = optionalTarget?.(optionalArgument);
  const binary = binaryLeft + binaryRight;
  const appended = appendReceiver.append(appendArgument);
  const member = memberReceiver[memberKey];
  return [dynamicObject, optional, binary, appended, member];
};
