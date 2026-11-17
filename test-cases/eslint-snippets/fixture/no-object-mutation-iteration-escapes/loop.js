function publishAcrossIterations(items, sink) {
  const rows = [];
  for (const item of items) {
    rows.push(item);
    sink(rows);
  }
  return rows;
}
