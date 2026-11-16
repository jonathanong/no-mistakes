function straightLine() {
  const values = [];
  values.push("kept local");
  return values;
}

function afterEarlyReturn(skip) {
  const values = [];
  if (skip) return values;
  values.push("kept local");
  return values;
}

function afterConditionalReturn(skip) {
  const values = [];
  if (skip) {
    return values;
  }
  values.push("kept local");
  return values;
}

function afterElseReturn(skip) {
  const values = [];
  if (skip) {
    return values;
  } else {
    values.push("kept local");
  }
  return values;
}

function returnFreshNested(skip) {
  if (skip) return { values: [] };
  const result = { values: [] };
  const values = result.values;
  values.push("kept local");
  return result;
}

function returnOptionalFresh(skip) {
  if (skip) return [];
  const values = [];
  values?.push("kept local");
  return values;
}

function returnBeforeAllocation(skip) {
  if (skip) return [];
  const values = [];
  values.push("kept local");
  return values;
}
