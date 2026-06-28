interface Options {
  value?: string | null;
  alias?: (string | null);
}

export function nullish(options: Options) {
  return options.value ?? "fallback";
}

export function orDefault(options: Options) {
  return options.alias || "fallback";
}

export function assignment(options: Options) {
  options.value ??= "fallback";
  options.alias ||= "fallback";
}

export function destructured(options: Options) {
  const { value = "fallback", alias = "fallback" } = options;
  return [value, alias];
}

export function paramDestructured({ value = "fallback" }: Options) {
  return value;
}
