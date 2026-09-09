class Service {
  static start() {
    this.load();
  }

  static async load() {
    await import("./this-member-loaded.mts");
  }
}

Service.start();
