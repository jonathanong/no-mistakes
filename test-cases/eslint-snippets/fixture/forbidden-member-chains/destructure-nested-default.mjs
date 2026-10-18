// A default inside a nested pattern is a receiver at any depth.
{
  const { x: { open } = other.panel } = {};
}
{
  const { a: { b: { open: show } = other.panel } = fallback } = {};
}
function nested({ panel: { open } = other.panel }) {
  return open;
}
{
  // A value default is not a nested pattern.
  const { open = other.panel } = {};
}
{
  const { y: { hidden } = missing } = other;
}
