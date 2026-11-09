function factory() {
  return function Value() {};
}
function other(value: number) {
  return value;
}
function chosen() {
  return (value: number) => value;
}

export function gaps() {
  (true).toFixed(1);
  const constructed = new (factory());
  constructed.toFixed(1);
  const sink: ({ join(separator: string): string }) = {
    join(separator: string) {
      return separator;
    },
  };
  sink.join(',');
  const qualified: Foo.Bar = 1 as unknown as Foo.Bar;
  qualified.toFixed(1);
  const mixed: string | number = 1;
  mixed.toFixed(1);

  const rows = [1, 2];
  const alias = rows!;
  alias.join(',');
  rows.forEach(function (item) {
    function nested(...rest: number[]) {
      return rest.length;
    }
    return nested(item);
  });
  const callback = other;
  rows.forEach(callback);
  rows.map(chosen());
}
