/** @typedef {{ data?: string }} Context */

/** @this {Context} */
export function declared() {
  this.data = 'ready';
  (() => { this.data = 'done'; })();
}

export const extensions = {
  /** @this {Context} */
  update() {
    this.data = 'ready';
    (() => { this.data = 'done'; })();
  },
  updateExpression: /** @this {Context} */ function () {
    this.data = 'expression';
  },
};

/** @this {Context} */
export const assigned = function () {
  this.data = 'assigned';
};
