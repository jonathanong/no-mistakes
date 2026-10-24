type Options = { label?: string | null };

export function member(options: Options) {
  const label = options.label;
  const alias = label;
  const chained = alias;
  return [alias ?? "Untitled", chained || "Untitled"];
}

export function destructured(options: Options) {
  const { label } = options;
  const alias = label;
  return alias ?? "Untitled";
}

export function nested(options: Options) {
  const label = options.label;
  {
    const alias = label;
    return alias || "Untitled";
  }
}
