class Client {
  append(row) {
    const push = this.push;
    push(row);
  }
}
function plain(row) {
  const push = this.push;
  push(row);
}
function dotted(rows, row) {
  const push = rows.allowed.push;
  push(row);
}
