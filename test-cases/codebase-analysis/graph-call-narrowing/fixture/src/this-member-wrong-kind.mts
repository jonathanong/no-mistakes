class Service {
  run() {
    this.load();
  }

  static load() {}

  static start() {
    this.run();
  }
}

new Service().run();
Service.start();
