export function builders(rows: any[], shared: any, publish: any) {
  rows.reduce<Record<string, unknown>>((acc, row) => {
    acc[row.key] = row;
    return acc;
  }, {});
  rows.reduceRight(function (acc, row) {
    acc.push(row);
    return acc;
  }, []);
  rows.reduce((acc, row) => {
    const alias = acc;
    alias[row.key] = row;
    if (row.skip) return acc;
    return acc;
  }, {} as Record<string, any>);

  // A literal only owns its container; shared nested values stay shared.
  rows.reduce((acc) => {
    acc.child.value = 1; // report: nested shared value
    return acc;
  }, { child: shared });
  rows.reduce((acc, row) => {
    publish(acc);
    acc[row.key] = row; // report: escaped accumulator
    return acc;
  }, {});
  rows.reduce((acc, row) => {
    row.value = 1; // report: element parameter remains shared
    acc[row.key] = row;
    return acc;
  }, {});
  rows.reduce((acc) => {
    acc.value = 1; // report: prior iteration leaked the accumulator
    publish(acc);
    return acc;
  }, {});
  rows.reduce((acc) => {
    acc.child.value = 1; // report: an escaped child survives into later iterations
    publish(acc.child);
    return acc;
  }, { child: {} });
  rows.reduce((acc) => {
    acc.child.value = 1; // report: child created inside the callback escaped earlier
    acc.child = {};
    publish(acc.child);
    return acc;
  }, { child: {} });
  const localRows = [1];
  localRows.reduce((acc, value) => {
    acc.push(value);
    return acc;
  }, []);
  const externalRows = () => rows;
  externalRows().reduce((acc) => { acc.value = 1; return acc; }, {});
  const box = { child: {} };
  publish(box.child);
  box.child.value = 1; // report: publishing a member also escapes it outside reductions
  const result = rows.reduce((acc) => {
    acc.value = 1;
    return acc;
  }, {});
  publish(result);
  result.value = 2; // report: result escaped after reduction
}
