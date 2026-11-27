declare const id: string;

export function SaveButton(props: Record<string, unknown>) {
  return (
    <>
      <button {...{ "data-pw": id }}>Save</button>
      <button data-pw={id ?? "save"}>Save</button>
      <button data-testid="save">Save</button>
      <button {...props}>Save</button>
    </>
  );
}
