type Options = { label?: string | null };

export function member(options: Options) {
  const label = options.label;
  const alias = label;
  const chained = alias;
  return [alias === undefined ? "Untitled" : alias, chained === undefined ? "Untitled" : chained];
}

export function destructured(options: Options) {
  const { label } = options;
  const alias = label;
  return alias === undefined ? "Untitled" : alias;
}

export function nested(options: Options) {
  const label = options.label;
  {
    const alias = label;
    return alias === undefined ? "Untitled" : alias;
  }
}

export function parameter({ label }: Options) {
  // A typed destructured parameter is already a known nullable option value.
  const alias = label;
  return alias === undefined ? "Untitled" : alias;
}

export function renamedParameter({ label: text }: Options) {
  const alias = text;
  const chained = alias;
  return chained === undefined ? "Untitled" : chained;
}
