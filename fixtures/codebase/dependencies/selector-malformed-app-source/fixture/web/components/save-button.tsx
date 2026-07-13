// This malformed app source ensures demanded selector scans surface parse errors.
export function SaveButton() {
  return <button data-testid="save">Save;
}
