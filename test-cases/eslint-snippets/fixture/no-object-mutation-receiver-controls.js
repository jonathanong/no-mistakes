/** @this {Object} */
export function declared() {
  function nested() {
    this.data = 'nested';
    (() => { this.data = 'nested arrow'; })();
  }
  return nested;
}

export const methods = {
  /* @this {Object} */
  ordinary() { this.data = 'ordinary block'; },
  // @this {Object}
  line() { this.data = 'line'; },
  /** @returns {Object} */
  returned() { this.data = 'return type'; },
  /** @this {} */
  empty() { this.data = 'empty'; },
  // @this {Object}
  /** @returns {Object} */
  mixed() { this.data = 'mixed comments'; },
};

/** @this {Object} */
const unrelated = 1;
export function distant() { this.data = unrelated; }

export function arrowAnnotation() {
  /** @this {Object} */
  return () => { this.data = 'arrow'; };
}

export function trailing() {} /** @this {Object} */
export function afterTrailing() { this.data = 'trailing'; }
