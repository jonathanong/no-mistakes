export function buildDynamicCsp(dynamicPart: string) {
  return [
    "default-src 'self'",
    dynamicPart,
    "img-src https://img.example.com",
  ].join("; ");
}
