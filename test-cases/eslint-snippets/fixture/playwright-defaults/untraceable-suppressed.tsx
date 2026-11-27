declare const id: string;

export function SaveButton() {
  return (
    <>
      {/* no-mistakes-disable-next-line playwright-defaults -- selector is supplied by a generated host */}
      <button {...{ "data-pw": id }}>Hidden</button>
      <button {...{ "data-pw": id }}>Shown</button>
    </>
  );
}
