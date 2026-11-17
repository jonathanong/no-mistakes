function localLoop(items) {
  const rows = [];
  for (const item of items) rows.push(item);
  return rows;
}

function localCallback(items) {
  return items.map((item) => {
    const rows = [];
    rows.push(item);
    return rows;
  });
}

function earlyReturnLoop(items, skip) {
  const rows = [];
  for (const item of items) {
    if (skip) return rows;
    rows.push(item);
  }
  return rows;
}
