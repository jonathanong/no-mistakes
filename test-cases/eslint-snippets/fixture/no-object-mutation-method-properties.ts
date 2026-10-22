import { importedRows } from "./rows";
const modulePush = importedRows.push;
modulePush.ready = true;
function properties(rows) {
  rows.push.ready = true;
  const push = rows.push;
  push.ready = true;
  for (const item of Object.values(push)) item.ready = true;
  const [first] = push;
  first.ready = true;
  const holder = [push];
  const [stored] = holder;
  stored.ready = true;
  push.custom(); // An unrelated property is not the captured mutator.
}

function joined(shared, flag) {
  const local = [];
  const left = flag ? shared.push : local.push;
  left.ready = true;
  const right = flag ? local.push : shared.push;
  right.ready = true;
}
