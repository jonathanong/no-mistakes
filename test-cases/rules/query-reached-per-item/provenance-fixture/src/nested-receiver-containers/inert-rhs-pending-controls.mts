// Older pending siblings must remain unvisited after a newer call rejects.
const controls = [[], {}, [1, [2, 3], 4], { a: 1, b: [2, 3] },
  [1, stop()], [stop(), 1], [,,1], [...spread], { ...spread },
  { get x() { return 1; } }, { set x(v) {} }, { m() {} },
  { [key]: 1 }, { __proto__: null }, function () {}, () => 1,
  true, null, 1, 1n, "x", /x/, (1 as number), identifier,
  (target = 1), (1, 2), class {}];
let box = { read() {} };
[box] = [box];
export { controls, box };
