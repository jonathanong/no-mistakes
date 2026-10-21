export function scalar(flag: boolean) {
  const first = flag ? ["left", [1, true]] : ["right", [0, false]];
  const second = flag ? ["active", [2, true]] : ["idle", [3, false]];
  return [first, second];
}
