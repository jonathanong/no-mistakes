export function register(value: string) { return value; }
// These calls must stay available to broad call-site consumers, but not finite-set demand.
export function Card() {
  return <article>{register("unrelated")}{register("also-unrelated")}</article>;
}
