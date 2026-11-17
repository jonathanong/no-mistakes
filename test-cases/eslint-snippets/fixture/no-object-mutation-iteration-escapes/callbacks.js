function mapped(items) {
  const rows = [];
  items.map(() => {
    rows.push("mapped");
    return rows;
  });
  rows.push("after map");
  return rows;
}

function visited(items) {
  const rows = [];
  items.forEach(() => {
    rows.push("visited");
    return rows;
  });
  rows.push("after forEach");
  return rows;
}
