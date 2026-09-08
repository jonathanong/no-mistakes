class Constructed {
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
