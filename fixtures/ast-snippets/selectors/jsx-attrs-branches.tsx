export function JsxAttrsBranches(props: { label?: string }) {
  return (
    <>
      <button data-case="label-bare" aria-label />
      <button data-case="label-string" aria-label="Name" />
      <button data-case="label-dynamic" aria-label={props.label} />
      <button data-case="label-null" aria-label={null} />
      <button data-case="label-undefined" aria-label={undefined} />
      <button data-case="label-satisfies" aria-label={props.label satisfies string | undefined} />
      <button data-case="hidden-bare" hidden />
      <button data-case="hidden-string" hidden="false" />
      <button data-case="hidden-null" hidden={null} />
      <button data-case="hidden-zero" hidden={0} />
      <button data-case="hidden-string-empty" hidden={""} />
      <button data-case="hidden-template" hidden={`visible`} />
      <button data-case="hidden-undefined" hidden={undefined} />
      <button data-case="hidden-as" hidden={true as boolean} />
      <button data-case="hidden-satisfies" hidden={false satisfies boolean} />
      <button data-case="hidden-non-null" hidden={value!} />
      <button data-case="hidden-dynamic" hidden={props.label} />
      <div data-case="aria-string-true" aria-hidden="true" />
      <div data-case="aria-string-false" aria-hidden="false" />
      <div data-case="aria-string-invalid" aria-hidden="mixed" />
      <div data-case="aria-expr-string-true" aria-hidden={"true"} />
      <div data-case="aria-expr-string-false" aria-hidden={"false"} />
      <div data-case="aria-expr-string-invalid" aria-hidden={"mixed"} />
      <div data-case="aria-as" aria-hidden={true as boolean} />
      <div data-case="aria-satisfies" aria-hidden={false satisfies boolean} />
      <div data-case="aria-non-null" aria-hidden={value!} />
      <select data-case="size-string" size="3" />
      <select data-case="size-number" size={4} />
      <select data-case="size-as" size={5 as const} />
      <select data-case="size-satisfies" size={6 satisfies number} />
      <select data-case="size-non-null" size={value!} />
      <select data-case="size-dynamic" size={props.label} />
      <select data-case="size-negative" size={-1} />
    </>
  );
}
