interface Options {
  value?: string | null;
  "literal-name"?: string | null;
}

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

export function ignoredBranches(options: Options) {
  const value = options.value;
  {
    const options = {};
    options.value ?? "fallback";
  }
  const unknown = {};
  return [value ?? "fallback", unknown.value ?? "fallback"];
}
