interface Options {
  value?: string | null;
  count?: number;
  required: string | null;
}

type Inline = {
  value?: string | null;
};

interface NullableBase {
  narrowed?: string | null;
}

interface NarrowedOptions extends NullableBase {
  narrowed: string;
}

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

export function catchShadow(options: Options) {
  try {
    return options.value === undefined ? "fallback" : options.value;
  } catch (options) {
    return options.value ?? "fallback";
  }
}

export function staleAssignmentCleared(options: Options) {
  let value = options.value;
  value = "forced";
  return value ?? "fallback";
}

export function objectAliasCleared(options: Options) {
  let opts = options;
  opts = {};
  return opts.value ?? "fallback";
}

export function inheritedNarrowed(options: NarrowedOptions) {
  return options.narrowed ?? "fallback";
}
