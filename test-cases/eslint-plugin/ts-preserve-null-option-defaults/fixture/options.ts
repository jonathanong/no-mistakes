interface Options {
  value?: string | null;
}

interface OtherConfig {
  value?: string | null;
}

type InlineOptions = {
  value?: string | null;
};

export function selected(options: Options) {
  return options.value ?? "fallback";
}

export function ignored(other: OtherConfig) {
  return other.value ?? "fallback";
}

export function inline(options: InlineOptions) {
  const { value = "fallback" } = options;
  return value;
}
