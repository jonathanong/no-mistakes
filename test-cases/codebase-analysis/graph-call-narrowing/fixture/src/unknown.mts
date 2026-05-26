async function hidden() {
  await import("./uncalled.mts");
}

runner[method]();
