class PrivateControls {
  #value = 0;
  static #shared = 0;
  constructor(value: number) {
    this.value = value;
    this.#shared = value;
    this.#value += value;
    const later = () => {
      this.#value = value;
    };
    later();
  }
  update(value: number) {
    this.#value = value;
  }
}
