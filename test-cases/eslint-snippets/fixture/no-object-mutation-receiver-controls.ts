interface Context { data?: string }

export const declared = {
  update(this: Context) {
    function nested() {
      this.data = 'nested';
      (() => { this.data = 'nested arrow'; })();
    }
    nested();
    const expression = function () { this.data = 'expression'; };
    expression();
  },
};

export const undeclared = {
  update() {
    this.data = 'method';
    (() => { this.data = 'arrow'; })();
  },
};

export function plain() {
  this.data = 'plain';
}
