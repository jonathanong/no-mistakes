class Service {
  constructor() {
    this.load();
  }

  async load() {
    await import("./this-member-loaded.mts");
  }
}

new Service();
