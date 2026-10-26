export function literal() {
  return "SELECT 1";
}
declare function external(): string;
export let changed = literal;
changed = external;
