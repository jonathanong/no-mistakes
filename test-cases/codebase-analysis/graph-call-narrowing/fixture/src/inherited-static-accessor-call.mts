class Base {
  static get value() {
    return 1;
  }
}

class Child extends Base {
  static value() {
    return 2;
  }
}

function run() {
  Child.value();
  Child.value;
}

run();
