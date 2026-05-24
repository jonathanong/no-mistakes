function Button(props: { testId: string; title: string; children: React.ReactNode }) {
  return <button data-pw={props.testId} title={props.title}>{props.children}</button>;
}

export function Example(props: { label: string }) {
  return (
    <>
      <label data-pw="email-label"> Email address </label>
      <input data-pw={"search-input"} aria-label={"Search field"} placeholder={"Search"} />
      <img alt="Company logo" />
      <Button testId="save-button" title="Save changes">Save</Button>
      <div data-pw={props.label}>Dynamic selector</div>
      <div {...props}>Spread child</div>
      <div data-pw="string-child">{"String child"}</div>
      <div>{props.label}</div>
    </>
  );
}
