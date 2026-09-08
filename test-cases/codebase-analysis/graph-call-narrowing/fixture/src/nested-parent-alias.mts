function target() {
  import("./nested-parent-alias-target.mts");
}

function outer() {
  const outerLoad = target;
  function inner() {
    // The inner alias must continue through its lexical parent binding.
    const load = outerLoad;
    load();
  }
  inner();
}

outer();
