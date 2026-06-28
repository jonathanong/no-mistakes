interface Options {
  value?: string | null;
  "literal-name"?: string | null;
}

type MaybeLabel = string | null;
type MaybeAlias = MaybeLabel;

interface AliasedOptions {
  label?: MaybeAlias;
}

interface MergedOptions {
  first?: string | null;
}

interface MergedOptions {
  second?: string | null;
}

export default interface DefaultOptions {
  defaulted?: string | null;
}

interface BaseOptions {
  inherited?: string | null;
}

interface ExtendedOptions extends BaseOptions {
  own?: string | null;
}

type PublicOptions = Options;
type IntersectedOptions = BaseOptions & {
  intersected?: string | null;
};

export function optionalMember(options: Options) {
  return options?.value ?? "fallback";
}

export function computedMember(options: Options) {
  return options["literal-name"] ?? "fallback";
}

export const expression = function (options: Options) {
  return options.value ?? "fallback";
};

export const arrow = (options: Options) => options.value || "fallback";

export function varScope() {
  if (Math.random()) {
    var options: Options = {};
  }
  return options.value ?? "fallback";
}

export function assignmentParam(options: Options = {}) {
  return options.value ?? "fallback";
}

export function destructuredTyped() {
  const { value = "fallback", ...rest }: Options = {};
  return [value, rest];
}

export function destructuredThenDefault(options: Options) {
  const { value } = options;
  return value ?? "fallback";
}

export function memberThenDefault(options: Options) {
  const value = options.value;
  return value || "fallback";
}

export function assertedObject() {
  const options = {} as Options;
  return options.value ?? "fallback";
}

export function assertedDestructure() {
  const { value } = {} as Options;
  return value ?? "fallback";
}

export function typeAssertionDestructure() {
  const { value } = <Options>{};
  return value ?? "fallback";
}

export function merged(options: MergedOptions) {
  return [options.first ?? "fallback", options.second ?? "fallback"];
}

export function defaultExported(options: DefaultOptions) {
  return options.defaulted ?? "fallback";
}

export function aliased(options: AliasedOptions) {
  return options.label ?? "fallback";
}

export function inherited(options: ExtendedOptions) {
  return [options.inherited ?? "fallback", options.own ?? "fallback"];
}

export function destructuringAssignment(options: Options) {
  let value;
  ({ value = "fallback" } = options);
  return value;
}

export function inlineAlias(options: { label?: MaybeAlias }) {
  return options.label ?? "fallback";
}

export function unionOptions(options: Options | undefined = {}) {
  return options.value ?? "fallback";
}

export function typeAliasOptions(options: PublicOptions) {
  return options.value ?? "fallback";
}

export function objectAlias(options: Options) {
  const opts = options;
  return opts.value ?? "fallback";
}

export function nestedDestructuringAssignment(options: Options) {
  let value;
  if (Math.random()) {
    ({ value } = options);
  }
  return value ?? "fallback";
}

export function plainAssignment(options: Options) {
  let value;
  value = options.value;
  return value ?? "fallback";
}

export function assertedMemberDefault(options: Options) {
  return (options.value as string | null) ?? "fallback";
}

export function nonNullMemberDefault(options: Options) {
  return [options.value! ?? "fallback", options!.value ?? "fallback"];
}

export function readonlyOptions(options: Readonly<Options>) {
  return options.value ?? "fallback";
}

export function restAlias(options: Options) {
  const { ...opts } = options;
  return opts.value ?? "fallback";
}

export function intersected(options: IntersectedOptions) {
  return [options.inherited ?? "fallback", options.intersected ?? "fallback"];
}

export function loopScope(options: Options) {
  for (let options of [{}]) {
    options.value;
  }
  return options.value ?? "fallback";
}

export function switchScope(options: Options) {
  switch (Math.random()) {
    case 1: {
      const options = {};
      options.value;
      break;
    }
  }
  return options.value ?? "fallback";
}

export function ignoredBranches(options: Options) {
  const value = options.value;
  const other = "other";
  {
    const options = {};
    options.value ?? "fallback";
  }
  const unknown = {};
  return [value ?? "fallback", other ?? "fallback", missing ?? "fallback", unknown.value ?? "fallback"];
}
