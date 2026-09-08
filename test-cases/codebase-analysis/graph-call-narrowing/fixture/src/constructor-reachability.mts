class Constructed {
  field = import("./instance-field-called.mts");

  constructor() {
    import("./constructor-called.mts");
  }

  unused() {
    import("./constructor-unused-method.mts");
  }
}

new Constructed();

class PlainCalled {
  constructor() {
    import("./constructor-plain-call.mts");
  }
}

PlainCalled();

class Unused {
  field = import("./instance-field-unused.mts");
}

class Eager {
  static field = import("./static-field-eager.mts");
}
