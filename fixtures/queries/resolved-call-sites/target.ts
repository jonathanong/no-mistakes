export function used(value: unknown) { return value; }
export function local() {
  used(1);
  function nested(used: () => void) { used(); }
  nested(() => {});
}
export { used as exposed };
