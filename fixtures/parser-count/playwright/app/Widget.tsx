export function Widget() {
  void fetch("/api/save");
  return <button data-testid="save">Save</button>;
}
