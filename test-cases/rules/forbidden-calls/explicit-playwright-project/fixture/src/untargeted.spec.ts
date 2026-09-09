// Outside the explicit `custom` include. Named and `playwright: true` roots
// must not treat this as a forbidden-calls Playwright project file.
export function otherTimer() {
  setTimeout(() => {}, 1);
}
