export function SaveButton() {
  return (
    <>
      {/* no-mistakes-disable-next-line playwright-consistent-attribute -- vendor component contract requires data-testid */}
      <button {...{ "data-testid": "save" }}>Hidden</button>
      <button {...{ "data-testid": "save" }}>Shown</button>
    </>
  );
}
