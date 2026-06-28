interface BaseOptions {
  inherited?: string | null;
}

interface Options extends BaseOptions {
  value?: string | null;
}

interface OtherConfig {
  value?: string | null;
}

type InlineOptions = {
  value?: string | null;
};

interface InternalOptions {
  publicValue?: string | null;
}

type PublicOptions = InternalOptions;

export function selected(options: Options) {
  return [options.value ?? "fallback", options.inherited ?? "fallback"];
}

export function ignored(other: OtherConfig) {
  return other.value ?? "fallback";
}

export function inline(options: InlineOptions) {
  const { value = "fallback" } = options;
  return value;
}

export function publicAlias(options: PublicOptions) {
  return options.publicValue ?? "fallback";
}
