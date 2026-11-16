function localAppend() {
  const rows = [];
  function append() {
    return rows.push("first");
  }
  append();
  rows.push("second");
  return rows;
}

function localConditionalAppend(flag) {
  const rows = [];
  function append() {
    return flag ? rows.push("first") : rows.unshift("first");
  }
  append();
  rows.push("second");
  return rows;
}

function localLogicalAppend(flag) {
  const rows = [];
  function append() {
    return flag && rows.push("first");
  }
  append();
  rows.push("second");
  return rows;
}
