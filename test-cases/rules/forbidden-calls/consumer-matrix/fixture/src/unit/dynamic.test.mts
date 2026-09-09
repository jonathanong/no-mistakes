export function unknownCall(
  runner: Record<string, () => void>,
  name: string,
) {
  runner[name]();
}
