function outer() {
  function a() {
    import("./called.mts");
  }

  function b() {
    a();
  }

  b();
}

outer();
