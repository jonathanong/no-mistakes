export function render(options: { limit?: number | null }) {
  return (options as { limit?: number | null }).limit ?? 10;
}

export function renderNonNull(options: { limit?: number | null }) {
  return (options!.limit satisfies number | null | undefined) ?? 10;
}

export function renderDestructure(options: { limit?: number | null }) {
  const { limit } = options!;
  return (limit as number | null | undefined) ?? 10;
}

export function renderOptional(options: { limit?: number | null }) {
  return options?.limit ?? 10;
}
