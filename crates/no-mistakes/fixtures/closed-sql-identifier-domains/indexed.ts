const VALUES = ["painted", "plain"] as const;
const WRAPPED = (["painted", "plain"] as const);
const CHECKED = ["painted", "plain"] as const satisfies readonly string[];
const WIDE: readonly string[] = ["painted", "plain"] as const;
type Variety = (typeof VALUES)[number];
type Alias = Variety;
export function types(
  finite: Variety,
  direct: (typeof VALUES)[number],
  alias: Alias,
  wrapped: (typeof WRAPPED)[number],
  checked: (typeof CHECKED)[number],
  parenthesized: ((typeof VALUES)[number]),
  wide: (typeof WIDE)[number],
  open: string,
) {}
export function shadow(VALUES: string[], shadowed: (typeof VALUES)[number]) {}
export function guarded(guardedName: string) {
  if (!VALUES.some(item => item === guardedName)) throw new Error("Unknown variety");
}
export function literalOnly(unbounded: string) {
  if (!VALUES.some(item => item === "painted")) throw new Error("Unknown variety");
}
