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
      <div data-pw="joined-text">Hello {"World"}<span>ignored</span>{"Again"}</div>
      <UI.Button testId="member-button" title="Member save">Member</UI.Button>
      <Design.UI.Button testId="nested-member-button" title="Nested member">Nested</Design.UI.Button>
      <button id="html-id-button">HTML id</button>
      <div data-pw="explicit-role" role="button">Explicit role</div>
      <a data-pw="link-target" href="/docs">Docs</a>
      <h2 data-pw="heading-target">Heading</h2>
      <img data-pw="image-target" alt="Hero image" />
      <input data-pw="checkbox-target" type="checkbox" aria-label="Subscribe" />
      <input data-pw="radio-target" type="radio" aria-label="Pick one" />
      <input data-pw="range-target" type="range" aria-label="Volume" />
      <input data-pw="hidden-target" type="hidden" aria-label="Hidden token" />
      <select data-pw="select-target" aria-label="Country" />
      <textarea data-pw="textarea-target" aria-label="Message" />
      <div data-pw="split-expression">Before {props.label}</div>
      <div>{props.label}</div>
    </>
  );
}
