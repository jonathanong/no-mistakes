function callEscape(skip, sink) {
  const values = [];
  if (skip) return values;
  sink(values);
  values.push("escaped");
  return values;
}

const published = {};
function storageEscape(skip) {
  const values = [];
  if (skip) return values;
  published.values = values;
  values.push("escaped");
  return values;
}

function closureEscape(skip, sink) {
  const values = [];
  if (skip) return values;
  sink(() => values.push("captured"));
  values.push("escaped");
  return values;
}

function* yieldEscape(skip) {
  const values = [];
  if (skip) return values;
  yield values;
  values.push("yielded");
  return values;
}

function finallyClosureEscape(skip, sink) {
  const values = [];
  try {
    if (skip) return values;
  } finally {
    sink(() => values.push("captured in finally"));
  }
  values.push("escaped");
  return values;
}

function returnedClosure(skip) {
  const values = [];
  if (skip) return values;
  return () => values.push("captured by returned closure");
}

function escapedBeforeReturn(skip, sink) {
  const values = [];
  sink(values);
  if (skip) return values;
  values.push("escaped before the return");
  return values;
}
