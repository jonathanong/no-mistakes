class Service {
  run() {
    const go = () => {
      this.load();
    };
    go();
  }

  load() {}
}

class StaticService {
  static start() {
    const go = () => {
      this.load();
    };
    go();
  }

  static load() {}
}

new Service().run();
StaticService.start();
