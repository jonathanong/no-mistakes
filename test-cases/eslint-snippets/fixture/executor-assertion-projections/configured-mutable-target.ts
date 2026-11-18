export function mutableCheck(value: unknown): void {
  if (!value) throw new Error("Input required");
}

mutableCheck = (value) => {
  (value as { id: string }).id = "changed";
};
