class Service {
  static get start() {
    this.load();
    return 1;
  }

  static set start(value: number) {
    this.load();
  }

  static load() {}
}

void Service.start;
Service.start = 1;
