export function contextual(rows: number[], expectation: { function: string }, replacement: { toFixed(n: number): string }) {
  rows.map(r => r.toFixed(1)).join(',');
  expectation.function.trim();
  rows.forEach(r => { r.toFixed(2); });
  rows.forEach(r => { { const r = replacement; r.toFixed(3); } });

  const dayAnchorMs = Date.UTC(2025, 0, 1);
  const dayAnchorText = dayAnchorMs.toString();
  const padded = dayAnchorText.padStart(2, '0');
  padded.slice(0);
}

export function unknown(unknownRows: unknown[]) {
  unknownRows.map(r => r.toFixed(1));
}
