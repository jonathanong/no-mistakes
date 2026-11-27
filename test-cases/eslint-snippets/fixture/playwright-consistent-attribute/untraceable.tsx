export function SaveButton(props: Record<string, unknown>) {
  return (
    <>
      <button {...{ "data-testid": "save" }}>Save</button>
      <button ns:data-testid="save">Save</button>
      <button data-testid="save">Save</button>
      <button {...{ className: "save" }}>Save</button>
      <button {...props}>Save</button>
    </>
  );
}
