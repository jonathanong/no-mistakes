class Service {
  run() {
    const name = "load";
    this[name]();
  }

  load() {}
}
