function overwrittenCallback(skip) {
  const values = [];
  const result = { run: () => values.push("escaped") };
  if (skip) return result;
  result.run = null;
  values.push("local");
  return null;
}

function reassignedCapture(skip) {
  let values = [];
  const local = values;
  if (skip) return () => values.push("escaped");
  values = null;
  local.push("local");
  return null;
}

function laterCallbackEscape(skip, sink) {
  const values = [];
  const callback = () => values.push("captured");
  if (skip) return callback;
  sink(callback);
  values.push("escaped");
  return values;
}

function overwrittenNestedCallback(skip) {
  const values = [];
  const result = { nested: { run: () => values.push("escaped") } };
  if (skip) return result;
  result.nested.run = null;
  values.push("local");
  return null;
}

function overwrittenBoundMethod(skip, sink) {
  const rows = [];
  function getResult() {
    const result = { run: rows.push.bind(rows) };
    if (skip) return result;
    result.run = null;
    return null;
  }
  sink(getResult());
  rows.push("escaped");
}

function returnedMap(sink) {
  const rows = new Map();
  function getRows() {
    return rows;
  }
  sink(getRows());
  rows.get("item").value = "escaped";
}
