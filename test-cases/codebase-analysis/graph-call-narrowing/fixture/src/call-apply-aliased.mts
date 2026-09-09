async function target() {
  await import("./call-apply-loaded.mts");
}

const alias = target;
alias.call(undefined);
