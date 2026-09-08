class Service {
  static run() {
    helper();

    function helper() {
      import("./called.mts");
    }
  }

  static unused() {
    import("./uncalled.mts");
  }
}

Service.run();
