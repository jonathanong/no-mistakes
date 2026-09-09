class Alpha extends Beta {
  run() {
    this.missing();
  }
}

class Beta extends Alpha {}

new Alpha().run();
