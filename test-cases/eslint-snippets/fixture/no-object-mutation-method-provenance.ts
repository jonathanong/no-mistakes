function escaped(sink) {
  const rows = [];
  const push = rows.push;
  sink(push);
  rows.push(1);
}
function boundEscape(sink) {
  const rows = [];
  const push = rows.push.bind(rows);
  sink(push);
  rows.push(1);
}
function capture(sink) {
  const rows = [];
  const push = rows.push;
  sink(() => push(1));
}
function reassigned(rows) {
  let local = [];
  const push = local.push;
  local = rows;
  push(1); // The captured receiver is still the original fresh array.
}
function preserved(rows, sink) {
  const local = [];
  const push = local.push;
  sink(local);
  push(1);
}
function actualMethod(rows) {
  const local = { push() { rows.ready = true; } };
  const push = local.push;
  push(); // A known local function keeps its own mutation body.
}
function returning() {
  const rows = [];
  return rows.push;
}
function returnBound() {
  const rows = [];
  return rows.push.bind(rows);
}

function boundShared(rows) {
  const local = [];
  const push = rows.push.bind(rows, 1);
  push.call(local);
  push.call(); // A bound function ignores an omitted thisArg as well.
  push.apply(local, []);
  const rebound = push.bind(local);
  rebound();
}
function boundValue(rows, sink) {
  const value = {};
  const push = rows.push.bind(rows, value);
  push();
  value.ready = true;
}
function escapedBoundValue(sink) {
  const local = [];
  const value = {};
  sink(local.push.bind(local, value));
  value.ready = true;
}

function capturedBoundValue(rows, sink) {
  const value = {};
  const push = rows.push.bind(rows, value);
  sink(() => push());
  value.ready = true;
}

function objectEscape(sink) {
  const local = [];
  const push = local.push;
  sink({ push });
  local.ready = true;
}
function boundObjectEscape(sink) {
  const local = [];
  const value = {};
  const push = local.push.bind(local, value);
  sink({ push });
  local.ready = true;
  value.ready = true;
}

function boundArrayEscape(sink) {
  const local = [];
  const value = {};
  const push = local.push.bind(local, value);
  sink([push]);
  local.ready = true;
  value.ready = true;
}
