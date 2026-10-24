export function controls(rows: any[], shared: any, publish: any) {
  rows.reduce((acc) => {
    acc.value = 1; // report: parameter seed
    return acc;
  }, shared);
  const initial = {};
  rows.reduce((acc) => {
    acc.value = 1; // report: identifier seeds are intentionally conservative
    return acc;
  }, initial);
  rows.reduce((acc) => {
    acc.value = 1; // report: implicit shared first element
    return acc;
  });
  rows.reduce((acc) => {
    acc.value = 1; // report: replacement result
    return {};
  }, {});
  rows.reduce((acc) => {
    acc.value = 1; // report: no returned accumulator
  }, {});
  rows.reduce((acc, row) => {
    if (row.skip) return {};
    acc.value = 1; // report: alternate return
    return acc;
  }, {});
  rows.reduce((acc) => {
    acc = shared;
    acc.value = 1; // report: rebound accumulator
    return acc;
  }, {});
  rows.reduce(async (acc) => {
    acc.value = 1; // report: promises do not preserve accumulator identity
    return acc;
  }, {});
  rows.reduce((acc) => {
    publish(() => { acc.value = 1; }); // report: escaped closure
    return acc;
  }, {});
  rows.reduce((acc) => {
    [acc] = [shared];
    acc.value = 1; // report: destructured rebinding
    return acc;
  }, {});
  rows.reduce((acc) => {
    function replace() { acc = shared; }
    replace();
    acc.value = 1; // report: a closure can replace the accumulator
    return acc;
  }, {});
  rows.reduce((acc) => {
    try { publish(); } catch (acc) { publish(acc); }
    acc.value = 1; // report: shadowed accumulator stays conservative
    return acc;
  }, {});
  rows.reduce((acc) => {
    for (acc of rows) { publish(acc); }
    acc.value = 1; // report: loop rebinding
    return acc;
  }, {});
  rows.reduce((acc) => {
    publish([, 1]);
    return; // no stable accumulator
  }, {});
  rows.reduce((acc) => acc, {});
  rows.reduce(() => shared, {});
  rows.reduce(function* (acc) {
    acc.value = 1; // report: generator does not return accumulator synchronously
    return acc;
  }, {});
  rows.reduce("callback", {});
  rows.reduce((acc) => { acc.value = 1; return acc; }, {}, 1); // report: unsupported arity
  rows.reduce((acc) => { acc.value = 1; return acc; }, ...rows); // report: spread seed
  const custom = { reduce(callback, seed) { return callback(shared); } };
  custom.reduce((acc) => { acc.value = 1; return acc; }, {}); // report: known custom method
  ({ reduce(callback, seed) { return callback(shared); } }).reduce((acc) => {
    acc.value = 1; // report: object method is not native array reduction
    return acc;
  }, {});
  const customField = { reduce: (callback) => callback(shared) };
  customField.reduce((acc) => { acc.value = 1; return acc; }, {}); // report: custom data field
  const box = { rows: { reduce(callback, seed) { return callback(shared); } } };
  box.rows.reduce((acc) => { acc.value = 1; return acc; }, {}); // report: nested custom method
  class Rows { reduce(callback, seed) { return callback(shared); } }
  const customRows = new Rows();
  customRows.reduce((acc) => { acc.value = 1; return acc; }, {}); // report: class method
}
