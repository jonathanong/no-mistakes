export function imported(sql: string) {
  const slots = arguments;
  return () => slots[0];
}
