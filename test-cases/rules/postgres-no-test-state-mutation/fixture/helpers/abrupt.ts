// The exported body, rather than its name or TypeScript return type, proves abrupt completion.
export function abortUnsupported(value: string): never {
  throw new Error(value);
}
