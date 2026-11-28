export function createSuite(_options: { baseURL: string }) {
  return { describe: { configure(_settings: object) {} } };
}
