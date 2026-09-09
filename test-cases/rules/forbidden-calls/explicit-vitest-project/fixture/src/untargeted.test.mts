// Outside the explicit `custom` include. Named and `vitest: true` roots must
// not treat this as a forbidden-calls Vitest project file.
export function otherTimer() {
  setTimeout(() => {}, 1);
}

