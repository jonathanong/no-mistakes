export function collect(segments: string[]): Set<string> {
  const seen = new Set<string>();
  const apps = new Set<string>();
  for (const segment of segments) {
    seen.add(segment);
  }
  apps.add(segments[1]);
  return new Set([...seen, ...apps]);
}
