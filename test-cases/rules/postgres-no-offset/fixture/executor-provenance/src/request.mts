export function createRequest() {
  return { get: (_path: string) => ({ query: (_params: object) => Promise.resolve() }) }
}
