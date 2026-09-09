export default class Service {
  constructor() {
    import("./named-default-class-dep.mts");
  }

  unused() {
    import("./named-default-class-unused.mts");
  }
}

new Service();
