async function target() {
  await import("./call-apply-loaded.mts");
}

function factory() {
  return target;
}

function unused() {
  factory().call(undefined);
}
