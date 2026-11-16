export class Controls {
  value: unknown;
  items: unknown[];
  static shared: unknown;
  constructor(value: unknown) {
    class Nested {
      field = (this.value = value);
      static { this.value = value; }
    }
    consume(Nested);
    this.missing = value;
    (() => { this.value = value; })();
    queueMicrotask(() => { this.value = value; });
    function nested() { this.value = value; }
    consume(nested);
    this.items.push(value);
    Object.assign(this, { value });
    this.value += 1;
    this.value++;
    this.shared = value;
    this.method = value;
    this.setter = value;
    this.items[0] = value;
  }
  method(value: unknown) { this.value = value; }
  set setter(value: unknown) { this.value = value; }
}
class Parent { inherited: unknown; }
export class Child extends Parent {
  constructor(value: unknown) {
    super();
    this.inherited = value;
  }
}
