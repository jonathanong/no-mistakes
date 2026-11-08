import { readBoundary } from './helper';
export function Panel({ rows }: { rows: Array<{ id: string; label: string }> }) {
  const selected = rows.filter(row => row.id.length > 2).map(row => ({ ...row, label: row.label.trim() }));
  return <section aria-label="results">{selected.map(row => <button data-testid={row.id} onClick={() => readBoundary(row.id)}><strong>{row.label}</strong><span>{row.id}</span></button>)}</section>;
}
export const renderCard = (name: string) => {
  const normalize = (value: string) => value.trim().toLowerCase();
  return <article><header>{normalize(name)}</header><main>{[name, name + '!'].map(value => <p>{normalize(value)}</p>)}</main></article>;
};
