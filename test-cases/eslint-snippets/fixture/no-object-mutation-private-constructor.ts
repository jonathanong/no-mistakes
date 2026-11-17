class PrivateFields {
  #other: unknown;
  #value: unknown;
  value: unknown;
  constructor(value: unknown) {
    this.#value = value;
    this.value = value;
  }
}

const PrivateExpression = class {
  #value: unknown;
  constructor(value: unknown) {
    this.#value = value;
  }
};
