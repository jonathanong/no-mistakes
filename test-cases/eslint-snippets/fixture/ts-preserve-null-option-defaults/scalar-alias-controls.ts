import { external } from "./scalar-alias-import";
type Options = { label?: string | null };

export function mutable(options: Options) {
  const label = options.label;
  // Mutable aliases are outside this bounded const-only proof.
  let alias = label;
  var other = label;
  return [alias ?? "Untitled", other || "Untitled"];
}

export function shadow(options: Options) {
  const label = options.label;
  {
    const label = "known";
    const alias = label;
    return alias ?? "Untitled";
  }
}

export function cleared(options: Options) {
  let label = options.label;
  label = "known";
  const alias = label;
  return alias ?? "Untitled";
}

export function forward(options: Options) {
  // A forward reference is not a proven scalar alias.
  const alias = label;
  const label = options.label;
  return alias ?? "Untitled";
}

export function unknown() {
  const alias = external;
  return alias ?? "Untitled";
}

export function assignment(options: Options) {
  const label = options.label;
  let alias;
  alias = label;
  return alias ?? "Untitled";
}

export function wrapped(options: Options) {
  const label = options.label;
  const alias = label as string | null;
  return alias ?? "Untitled";
}

export function hoisted(options: Options) {
  const label = options.label;
  function inner() {
    // The inner hoisted variable owns this read, not the nullable outer binding.
    const alias = label;
    var label;
    return alias ?? "Untitled";
  }
  return inner;
}

export function temporal(options: Options) {
  const label = options.label;
  {
    const alias = label;
    const label = "known";
    return alias ?? "Untitled";
  }
}

export function mutableSource(options: Options) {
  let label = options.label;
  const alias = label;
  return alias ?? "Untitled";
}

export function changedConst(options: Options) {
  // Invalid TypeScript writes still invalidate an alleged immutable alias source.
  const label = options.label;
  label = "known";
  const alias = label;
  return alias ?? "Untitled";
}

export function parameter({ label }: Options) {
  // Writes invalidate a parameter source even if its annotation admits null.
  label = "known";
  const alias = label;
  return alias ?? "Untitled";
}

export function unbound() {
  const alias = missing;
  return alias ?? "Untitled";
}

export function laterParameterWrite({ label }: Options) {
  const alias = label;
  label = "known";
  return alias ?? "Untitled";
}

export function shadowedParameter({ label }: Options) {
  return (label: string) => {
    const alias = label;
    return alias ?? "Untitled";
  };
}

// Defaults execute before later parameters are initialized: do not follow them.
export function parameterDefault(
  unused = (() => {
    const alias = label;
    return alias ?? "Untitled";
  })(),
  { label }: Options,
) {}
