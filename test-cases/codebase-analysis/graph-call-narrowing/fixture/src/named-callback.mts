async function helper() {
  await import("./called.mts");
}

setTimeout(helper, 0);
