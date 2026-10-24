// no-mistakes-disable-file ts-preserve-null-option-defaults -- legacy API treats null as omission
type Options = { label?: string | null };
export function render(options: Options) {
  const label = options.label;
  const alias = label;
  return alias ?? "Untitled";
}
