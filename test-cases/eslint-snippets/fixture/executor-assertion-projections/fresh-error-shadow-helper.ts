const Object = {
  assign(target: Error, source: unknown) {
    return target;
  },
};

export function shadowedObject(value: unknown, status: number): asserts value {
  if (!value) throw Object.assign(new Error("assertion failed"), { status });
}
