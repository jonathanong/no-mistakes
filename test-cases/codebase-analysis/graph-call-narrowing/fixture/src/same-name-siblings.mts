function outer() {
  {
    function f() {
      import("./sibling-first.mts");
    }
    f();
  }
  {
    function f() {
      import("./sibling-second.mts");
    }
  }
}

outer();
