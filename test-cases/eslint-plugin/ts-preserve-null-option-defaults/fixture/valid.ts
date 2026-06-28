interface Options {
  value?: string | null;
  count?: number;
  required: string | null;
}

type Inline = {
  value?: string | null;
};

export function explicitUndefined(options: Options) {
  return options.value === undefined ? "fallback" : options.value;
}

export function destructured(options: Options) {
  const { value = "fallback", alias = "fallback" } = options;
  return [value, alias];
}

export function paramDestructured({ value = "fallback" }: Options) {
  return value;
}

export function nonNullable(options: Options) {
  return options.count ?? 0;
}

export function requiredNullable(options: Options) {
  return options.required ?? "fallback";
}

export function destructuringWithoutDefault(options: Options) {
  const { value } = options;
  return value;
}

export function inlineOk(options: Inline) {
  return options.value === undefined ? "fallback" : options.value;
}
