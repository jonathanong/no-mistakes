class Service {
  run() {
    this.load();
  }

  static run() {
    this.load();
  }

  load() {}

  static load() {}
}

new Service().run();
Service.run();
