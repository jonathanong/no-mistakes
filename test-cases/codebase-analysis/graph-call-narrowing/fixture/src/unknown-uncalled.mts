async function loader() {
  await import("./called.mts");
}

async function hidden() {
  runner[method]();
}
