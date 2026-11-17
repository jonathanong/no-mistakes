function conditionalBoundMutator(sink, flag) {
  const rows = [];
  function getPush() {
    return flag ? rows.push.bind(rows) : null;
  }
  sink(getPush());
  rows.push("escaped");
}

function logicalBoundMutator(sink, flag) {
  const rows = [];
  function getPush() {
    return flag && rows.push.bind(rows);
  }
  sink(getPush());
  rows.push("escaped");
}

function alternateBoundMutator(sink, fallback) {
  const rows = [];
  function getPush() {
    return rows.push.bind(rows) || fallback;
  }
  sink(getPush());
  rows.push("escaped");
}

function returnedBoundArguments(sink, flag) {
  const rows = [];
  const child = [];
  function getPush() {
    return flag ? rows.push.bind(rows, child) : null;
  }
  sink(getPush());
  child.push("escaped");
}

function returnedClass(flag) {
  const rows = [];
  if (flag) return class {
    append() {
      rows.push("captured");
    }
  };
  rows.push("local");
  return rows;
}
