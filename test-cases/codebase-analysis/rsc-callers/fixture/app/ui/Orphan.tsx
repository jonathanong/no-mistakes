// Exists but is imported by nothing -> empty callers (not an error).
export function Orphan() {
  return <span>orphan</span>;
}
