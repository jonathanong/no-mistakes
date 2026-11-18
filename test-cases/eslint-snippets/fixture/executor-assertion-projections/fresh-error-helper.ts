export function check(value: unknown, status: number): asserts value {
  if (!value) throw Object.assign(new Error("assertion failed"), { status });
}

export function explicitMetadata(value: unknown, status: number): asserts value {
  if (!value)
    throw Object.assign(new Error("assertion failed"), {
      code: status,
      message: "Missing input",
      count: 1,
      retry: false,
      active: true,
      reason: null,
      404: "Missing input",
      "error-code": status,
    });
}

export function emptyMetadata(value: unknown, status: number): asserts value {
  if (!value) throw Object.assign(new Error("assertion failed"), {});
}
