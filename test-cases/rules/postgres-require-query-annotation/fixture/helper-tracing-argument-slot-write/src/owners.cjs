// These sloppy calls exercise canonical ownership without creating query events.
function direct(directParam) {
  const caseName = "direct";
  return directParam;
}
function duplicate(repeatedParam, repeatedParam) {
  const caseName = "duplicate";
  return repeatedParam;
}
function absentLast(lostParam, lostParam) {
  const caseName = "absent";
  return lostParam;
}
function noParameters() {
  const caseName = "empty";
  return arguments;
}
function outer(outerParam) {
  function inner(innerParam) {
    // A bare var shadows the inherited formal without replacing this call's own mapping.
    var outerParam;
    const caseName = "inner";
    return innerParam;
  }
  const result = inner(outerParam);
  const caseName = "outer";
  return result;
}
const directResult = direct("/* direct */ SELECT 1");
const duplicateResult = duplicate("first", "/* duplicate */ SELECT 1");
const absentResult = absentLast("first");
const emptyResult = noParameters("/* actual extra slot */ SELECT 1");
const nestedResult = outer("/* nested */ SELECT 1");
