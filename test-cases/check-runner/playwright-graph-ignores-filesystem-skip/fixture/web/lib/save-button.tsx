// This component is intentionally beneath a filesystem-only skipped directory.
export function SaveButton() {
  return <button data-testid="save">Save</button>;
}

// Keep one uncovered selector so the regression cannot pass when selector
// discovery silently omits this filesystem-skipped directory.
export function DeleteButton() {
  return <button data-testid="delete">Delete</button>;
}
