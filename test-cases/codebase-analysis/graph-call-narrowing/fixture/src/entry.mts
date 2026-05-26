async function called() {
  await import("./called.mts");
}

async function uncalled() {
  await import("./uncalled.mts");
}

called();
