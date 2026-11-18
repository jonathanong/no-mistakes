function sharedComputedKey(output, key) {
  output[key].push("shared");
}

function escapeThroughCall(consume) {
  const items = ["first", "second"];
  const pending = [];
  items.forEach((item) => {
    pending.push(item);
    consume(pending);
  });
}

function escapeThroughStorage(output) {
  const items = ["first", "second"];
  const pending = [];
  items.forEach((item) => {
    pending.push(item);
    output.pending = pending;
  });
}

function customMap(map, consume) {
  const collection = { map };
  const pending = [];
  collection.map((item) => {
    pending.push(item);
    return item;
  });
  consume(pending);
}

async function asyncCallback(wait, consume) {
  const items = ["first", "second"];
  const pending = [];
  const work = items.map(async (item) => {
    await wait();
    pending.push(item);
  });
  consume(pending);
  await Promise.all(work);
}

function unknownCallback(register, consume) {
  const pending = [];
  register((item) => {
    pending.push(item);
  });
  consume(pending);
}

function returnedFromCallback(consume) {
  const items = ["first", "second"];
  const pending = [];
  const results = items.map((item) => {
    pending.push(item);
    return pending;
  });
  consume(results);
}

function overriddenFreshArray(map, consume) {
  const items = ["first", "second"];
  items.map = map;
  const pending = [];
  items.map((item) => {
    pending.push(item);
    return item;
  });
  consume(pending);
}

function escapedReceiver(expose, consume) {
  const items = ["first", "second"];
  const pending = [];
  expose(items);
  items.map((item) => {
    pending.push(item);
    return item;
  });
  consume(pending);
}

function generatorCallback(consume) {
  const items = ["first", "second"];
  const pending = [];
  const iterators = items.map(function* collect(item) {
    pending.push(item);
    yield item;
  });
  consume(pending, iterators);
}

function spreadThisArg(args, consume) {
  const items = ["first", "second"];
  const pending = [];
  items.forEach((item) => {
    pending.push(item);
  }, ...args);
  consume(pending);
}

function excessArguments(consume) {
  const items = ["first", "second"];
  const pending = [];
  items.forEach((item) => {
    pending.push(item);
  }, undefined, "extra");
  consume(pending);
}

function callbackIdentifier(consume) {
  const items = ["first", "second"];
  const pending = [];
  const collect = (item) => {
    pending.push(item);
    return item;
  };
  items.map(collect);
  consume(pending);
}

function computedMethodOverride(key, custom, consume) {
  const items = ["first", "second"];
  items[key] = custom;
  const pending = [];
  items.map((item) => {
    pending.push(item);
    return item;
  });
  consume(pending);
}

function aliasedMethodOverride(custom, consume) {
  const items = ["first", "second"];
  const alias = items;
  alias.map = custom;
  const pending = [];
  items.map((item) => {
    pending.push(item);
    return item;
  });
  consume(pending);
}

function assignedMethodOverride(custom, consume) {
  const items = ["first", "second"];
  Object.assign(items, { map: custom });
  const pending = [];
  items.map((item) => {
    pending.push(item);
    return item;
  });
  consume(pending);
}

function sharedOnNextIteration(shared) {
  const items = ["first", "second"];
  let out = [];
  items.forEach((item) => {
    out.push(item);
    out = shared;
  });
}

function sharedOnThirdIteration(shared) {
  const items = ["first", "second", "third"];
  let first = [];
  let second = [];
  items.forEach((item) => {
    first.push(item);
    first = second;
    second = shared;
  });
}

function capturedContainerReplay(shared) {
  const box = { current: [] };
  ["first", "second"].forEach((item) => {
    box.current.push(item);
    box.current = shared;
  });
}

function capturedContainerNestedRead(shared) {
  const out = [];
  ["first", "second"].forEach((item) => {
    out.push(shared);
    const first = out[0];
    first.push(item);
  });
}

function uncertainReceiver(flag, make, consume) {
  const items = flag ? [] : make();
  const out = [];
  items.map((value) => {
    out.push(value);
    return value;
  });
  consume(out);
}

function uncertainLogicalReceiver(make, consume) {
  const items = make() || [];
  const out = [];
  items.map((value) => {
    out.push(value);
    return value;
  });
  consume(out);
}

function callbackObjectMutation(shared) {
  const box = { current: [] };
  [1, 2].forEach((value) => {
    box.current.push(value);
    Object.assign(box, { current: shared });
  });
}

function callbackPropertyUpdate(shared) {
  const pending = [];
  [1, 2].forEach((value) => {
    pending.push(value);
    shared.count++;
  });
}

function assignedArrayPrototype(retain, consume) {
  Object.assign(Array.prototype, { map: retain });
  const pending = [];
  [1, 2].map((value) => pending.push(value));
  consume(pending);
}

function overriddenArrayPrototype(retain, consume) {
  Array.prototype.forEach = retain;
  const pending = [];
  ["first", "second"].forEach((item) => {
    pending.push(item);
  });
  consume(pending);
}
