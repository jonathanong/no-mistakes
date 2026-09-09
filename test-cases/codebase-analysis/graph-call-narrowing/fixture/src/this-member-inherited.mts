class Base {
  load() {}
}

class Derived extends Base {
  run() {
    this.load();
  }
}
