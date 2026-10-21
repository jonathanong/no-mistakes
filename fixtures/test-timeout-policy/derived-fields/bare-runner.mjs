// A collection subclass with no onBeforeCollect: the guard must still protect
// on the public collection hook and return undefined.
export class BareRunner {
  constructor(config) {
    this.config = config;
    this.target = new.target;
  }
}
