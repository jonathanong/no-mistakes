declare const useFixture: boolean;
const other = { describe: { configure(_options: object) {} } };

export function choose<T>(fixture: T) {
  return useFixture ? fixture : other;
}

export function forward<T>(fixture: T) {
  return fixture;
}
