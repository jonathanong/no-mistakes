type Options = { label?: string | null };
export function suppressed(options: Options) {
  const label = options.label;
  const alias = label;
  // no-mistakes-disable-next-line ts-preserve-null-option-defaults -- explicit null is omission here
  const first = alias ?? "Untitled";
  const second = alias || "Untitled"; // no-mistakes-disable-line ts-preserve-null-option-defaults -- intentional
  return [first, second, alias ?? "Untitled"];
}

export function legacy(options: Options) {
  // no-mistakes-disable-next-line ts-preserve-null-option-defaults -- intentional old form
  return options.label ?? "Untitled";
}

export function throughSuppressedOrigin(options: Options) {
  // no-mistakes-disable-next-line ts-preserve-null-option-defaults -- keep facts from this declaration
  const label = options.label;
  const alias = label;
  return alias ?? "Untitled";
}
