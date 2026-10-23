function controls(rows, key, replacement) {
  const unknown = rows[key];
  unknown(1);
  const { push = replacement } = rows;
  push(1);
  const { ...rest } = rows;
  rest(1);
  let overwritten = rows.push;
  overwritten = replacement;
  overwritten(1);
  const original = rows.push;
  function shadowed(original) { original(1); }
  const local = [];
  const localPush = local.push;
  localPush(1);
  const value = {};
  original.call(localPush, value);
  value.ready = true;
  original.call(local, 1);
  original.apply(local, [1]);
  const localBound = original.bind(local);
  localBound(1);
  localBound.call();
  localBound.ready = true; // Binding creates a separate local function object.
  original.bind(null)(1);
  original.apply(...rows);
  original.call();
  original.apply(rows);
  original.apply(rows, replacement);
  original.apply(rows, [, 1]);
  original.apply(rows, [...rows]);
  original.bind()();
  original.bind(...rows)();
  const rebound = localBound.bind(rows, 1, ...local);
  rebound.call(rows, 1);
  rebound.call();
  rebound.apply(rows, [1]);
  const cycle = {};
  cycle.self = cycle;
  const method = cycle.push;
  method(1);
  Object.values(method);
  const spread = { ...method };
  void spread;
}
{
  const nestedRows = [];
  const nestedPush = nestedRows.push;
  nestedPush(1);
}
const ignoredGlobal = globalThis.push;
ignoredGlobal(1);
const ignoredWindow = window.push;
ignoredWindow(1);
const ignoredEnvironment = process.env.push;
ignoredEnvironment(1);
const ignoredModule = module.exports.push;
ignoredModule(1);

function moduleFunction() {}
const namedMethod = moduleFunction.push;
namedMethod();
