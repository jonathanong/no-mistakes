function target() {
  import("./nested-parent-alias-target.mts");
}

function outer() {
  const load = target;
  function inner() {
    load();
  }
  inner();
}

outer();
