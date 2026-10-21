interface Options {
  label?: string | null;
}

export function bracket(options: Options) {
  return options["label"] ?? "Untitled";
}

export function template(options: Options) {
  return options[`label`] ?? "Untitled";
}

export function named(options: Options) {
  const { [`label`]: text } = options;
  return text ?? "Untitled";
}

export function dynamic(options: Options, key: "label") {
  return options[key] ?? "Untitled";
}
