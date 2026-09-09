function target() {
  return import("./called.mts");
}

function beforeAlias() {
  alias();
  const alias = target;
}

function afterAlias() {
  const alias = target;
  alias();
}

function directBefore() {
  later();
  const later = () => import("./uncalled.mts");
}

function nestedAfterInit() {
  function nested() {
    const inner = later;
    inner();
  }
  const later = () => import("./called.mts");
  nested();
}

function nestedBeforeOnly() {
  function nested() {
    const inner = later;
    inner();
  }
  nested();
  const later = () => import("./uncalled.mts");
}

function nestedEarlyAndLate() {
  function nested() {
    const inner = later;
    inner();
  }
  nested();
  const later = () => import("./uncalled.mts");
  nested();
}

beforeAlias();
afterAlias();
directBefore();
nestedAfterInit();
nestedBeforeOnly();
nestedEarlyAndLate();
