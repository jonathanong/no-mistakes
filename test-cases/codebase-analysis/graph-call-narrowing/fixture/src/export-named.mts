async function exported() {
  await import("./called.mts");
}

async function hidden() {
  await import("./uncalled.mts");
}

export { exported };
