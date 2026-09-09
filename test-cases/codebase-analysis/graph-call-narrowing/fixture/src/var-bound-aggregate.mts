function run() {
  {
    var api = {
      load() {
        import("./var-bound-aggregate-dep.mts");
      },
      unused() {
        import("./var-bound-aggregate-unused.mts");
      },
    };
  }
  api.load();
}

run();
