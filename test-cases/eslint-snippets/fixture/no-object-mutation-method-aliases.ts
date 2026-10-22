import { importedRows } from "./rows";
const moduleRows = [];
const push = moduleRows.push;
const again = push;
again(1);
const importedPush = importedRows["push"];
importedPush?.(2);
function write(rows, more) {
  const pop = rows[`pop`];
  pop();
  const { push: append } = rows;
  append(3);
  const { [`shift`]: shift } = rows;
  shift();
  const copied = append;
  copied.call(more, 4);
  copied["apply"](more, [5]);
  const bound = append.bind(more);
  bound(6);
}
