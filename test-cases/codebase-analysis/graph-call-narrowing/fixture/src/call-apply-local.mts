async function target() {
  await import("./call-apply-loaded.mts");
}

target.call(undefined);
