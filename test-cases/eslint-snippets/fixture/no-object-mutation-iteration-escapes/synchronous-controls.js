function mapThenConsume(consume) {
  const items = ["first", "second"];
  const pending = [];
  items.map((item) => {
    pending.push(item);
    return item;
  });
  consume(pending);
}

function forEachThenConsume(consume) {
  const items = ["first", "second"];
  const pending = [];
  items.forEach((item) => {
    pending.push(item);
  });
  consume(pending);
}

function flatMapThenConsume(consume) {
  const items = ["first", "second"];
  const pending = [];
  items.flatMap((item) => {
    pending.push(item);
    return [item];
  });
  consume(pending);
}

function filterThenConsume(consume) {
  const items = ["first", "second"];
  const pending = [];
  items.filter((item) => {
    pending.push(item);
    return item === "first";
  });
  consume(pending);
}

function earlyReturn(skip, consume) {
  const pending = [];
  if (skip) return pending;
  const items = ["first", "second"];
  items.forEach((item) => {
    pending.push(item);
  });
  consume(pending);
}

function copyAndDrain(source) {
  const pending = [...source];
  pending.shift();
  pending.push("new");
  return pending;
}

function literalArrayWithThisArg(consume) {
  const pending = [];
  ["first", "second"].forEach(function collect(item) {
    this.push(item);
  }, pending);
  consume(pending);
}

function aliasedArrayThenConsume(consume) {
  const items = ["first", "second"];
  const alias = items;
  const pending = [];
  alias.map((item) => {
    pending.push(item);
    return item;
  });
  consume(pending);
}

function shadowedArrayPrototype(retain, consume) {
  const Array = { prototype: {} };
  Array.prototype.forEach = retain;
  const pending = [];
  [1].forEach((value) => pending.push(value));
  consume(pending);
}
