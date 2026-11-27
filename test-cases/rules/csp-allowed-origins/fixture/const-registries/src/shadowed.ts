const origins = ["https://api.example.com", "https://cdn.example.com"] as const;

export function buildCsp() {
  return `connect-src 'self' ${origins.join(" ")}`;
}

export function build(origins: string[]) {
  return `connect-src 'self' ${origins.join(" ")}`;
}

export const buildArrow = (origins: string[]) =>
  `connect-src 'self' ${origins.join(" ")}`;

export function buildInner() {
  const origins = ["https://inner.example.net"];
  return `connect-src 'self' ${origins.join(" ")}`;
}

export function buildCatch() {
  try {
    throw new Error("miss");
  } catch (origins) {
    return `connect-src 'self' ${origins.join(" ")}`;
  }
}

export function buildFor(items: string[]) {
  for (const origins of items) {
    void origins;
  }
  for (const origins in items) {
    void origins;
  }
  for (let origins = items; origins.length > 0; origins = []) {
    void origins;
  }
}

{
  const origins = ["https://block.example.net"];
  void origins;
}

export class Holder {
  static {
    const origins = ["https://static.example.net"];
    void origins;
  }
}

export namespace Nested {
  const origins = ["https://nested.example.net"];
}
