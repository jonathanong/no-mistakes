// Max proves this uncached option; low deliberately reports its indirect shape.
const options = { cache: "no-store" };
export function uncached() {
  return fetch("/uncached", options);
}
