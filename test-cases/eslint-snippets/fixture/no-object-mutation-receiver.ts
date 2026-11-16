interface Context { cache?: Promise<string>; data?: string }
declare function load(key?: string): Promise<string>;

const extensions = {
  async getData(this: Context, key?: string) {
    if (this.cache) return this.cache;
    this.cache = (async () => {
      const value = await load(key);
      this.data = value;
      return value;
    })();
    return this.cache;
  },
};

export function apply(app: { extend(value: object): void }) {
  app.extend(extensions);
}

export function assign(app: { context: Context }) {
  Object.assign(app.context, {
    async getData(this: Context, key?: string) {
      this.cache = (async () => {
        this.data = await load(key);
        return this.data;
      })();
      return this.cache;
    },
  });
}

export function standalone(this: Context) {
  this.data = 'ready';
  (() => { this.data = 'done'; })();
}

export function nestedReceiver(this: Context) {
  this.data = 'outer';
  function nested(this: Context) {
    this.data = 'inner';
    (() => { this.data = 'arrow'; })();
  }
  return nested;
}

export const expression = function (this: Context) {
  this.data = 'expression';
};

export function localReceiver() {
  const local = {
    update(this: Context) { this.data = 'local'; },
  };
  local.update();
}
