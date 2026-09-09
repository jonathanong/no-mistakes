function run() {
  class Service extends loadBase() {
    static loaded = import("./called.mts");
    static {
      import("./called.mts");
    }
    [loadKey()]() {}
    method() {
      import("./uncalled.mts");
    }
  }
}

run();
