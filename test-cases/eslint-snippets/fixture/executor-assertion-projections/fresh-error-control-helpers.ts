declare function nextStatus(): number;

export function parameterTarget(value: unknown, status: number): asserts value {
  if (!value) throw Object.assign(value as object, { status });
}

export function propertyRead(value: unknown, status: number): asserts value {
  if (!value)
    throw Object.assign(new Error("assertion failed"), { status: (value as { id: string }).id });
}

export function captureInput(value: unknown, status: number): asserts value {
  if (!value) throw Object.assign(new Error("assertion failed"), { status, input: value });
}

export function getterSource(value: unknown, status: number): asserts value {
  if (!value)
    throw Object.assign(new Error("assertion failed"), {
      get status() {
        return status;
      },
    });
}

export function spreadSource(value: unknown, status: number): asserts value {
  if (!value) throw Object.assign(new Error("assertion failed"), { ...{ status } });
}

export function effectfulStatus(value: unknown, status: number): asserts value {
  if (!value) throw Object.assign(new Error("assertion failed"), { status: nextStatus() });
}

declare const metadata: { status: number };

export function computedKey(value: unknown, status: number): asserts value {
  if (!value) throw Object.assign(new Error("assertion failed"), { ["status"]: status });
}

export function methodSource(value: unknown, status: number): asserts value {
  if (!value)
    throw Object.assign(new Error("assertion failed"), {
      status() {
        return status;
      },
    });
}

export function aliasedMetadata(value: unknown, status: number): asserts value {
  if (!value) throw Object.assign(new Error("assertion failed"), metadata);
}

export function optionalAssign(value: unknown, status: number): asserts value {
  if (!value) throw Object.assign?.(new Error("assertion failed"), { status });
}

export function elementAssign(value: unknown, status: number): asserts value {
  if (!value) throw Object["assign"](new Error("assertion failed"), { status });
}
