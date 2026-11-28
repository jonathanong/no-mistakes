export function stillLoader(value: Promise<string>) {
  type require = unknown;
  const React = require("react");
  return React.use(Promise.resolve(value));
}

export function outerParameter(require: (id: string) => { use(value: unknown): unknown }) {
  {
    type require = unknown;
    const React = require("react");
    return React.use(Promise.resolve(1));
  }
}
