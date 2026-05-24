function Button(props: { testId: string; title: string; children: React.ReactNode }) {
  return <button data-pw={props.testId} title={props.title}>{props.children}</button>;
}

function InputWithDefaults(id = "identifier-email", selector = "identifier-email-input") {
  return (
    <>
      <label htmlFor={id}>Identifier email</label>
      <input id={id} data-pw={selector} />
      <button data-pw={`template-button`}>{`Template child`}</button>
      <input data-pw={`template-aria`} aria-label={`Template aria`} />
    </>
  );
}

export function Example(props: { label: string }) {
  return (
    <>
      <label data-pw="email-label"> Email address </label>
      <label htmlFor="named-email">Named email</label>
      <input id="named-email" data-pw="named-email-input" />
      <label htmlFor="subscribe-checkbox">Subscribe label</label>
      <input id="subscribe-checkbox" data-pw="subscribe-checkbox" type="checkbox" />
      <span id="plan-label">Plan label</span>
      <input id="plan-input" data-pw="plan-input" aria-labelledby="plan-label" />
      <input id="missing-labelledby-input" data-pw="missing-labelledby-input" aria-labelledby="missing-label" />
      <label htmlFor="missing-control">Missing control</label>
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
      <button data-pw="descendant-button"><span>Descendant save</span></button>
      <button data-pw="hidden-button" hidden>Hidden action</button>
      <button data-pw="hidden-false-button" hidden={false}>Shown action</button>
      <button data-pw="aria-hidden-button" aria-hidden="true">Aria hidden action</button>
      <button data-pw="aria-hidden-bool-button" aria-hidden={true}>Bool hidden action</button>
      <label>Wrapped email <input data-pw="wrapped-email-input" /></label>
      <div data-pw="container-target"><button>Container child</button></div>
      <input data-pw="submit-input" type="submit" value="Submit form" />
      <div data-pw="explicit-role" role="button">Explicit role</div>
      <a data-pw="link-target" href="/docs">Docs</a>
      <a data-pw="empty-link-target" href="">Empty docs</a>
      <a data-pw="dynamic-link-target" href={props.label}>Dynamic docs</a>
      <h2 data-pw="heading-target">Heading</h2>
      <img data-pw="image-target" alt="Hero image" />
      <input data-pw="checkbox-target" type="checkbox" aria-label="Subscribe" />
      <input data-pw="radio-target" type="radio" aria-label="Pick one" />
      <input data-pw="range-target" type="range" aria-label="Volume" />
      <input data-pw="searchbox-target" type="search" aria-label="Search site" />
      <input data-pw="number-target" type="number" aria-label="Count" />
      <input data-pw="hidden-target" type="hidden" aria-label="Hidden token" />
      <input data-pw="password-target" type="password" aria-label="Secret" />
      <select data-pw="select-target" aria-label="Country" />
      <select data-pw="listbox-target" multiple aria-label="Tags" />
      <select data-pw="multiple-false-target" multiple={false} aria-label="Single tag" />
      <select data-pw="sized-listbox-target" size="2" aria-label="Regions" />
      <select data-pw="numeric-sized-listbox-target" size={2} aria-label="Numeric regions" />
      <textarea data-pw="textarea-target" aria-label="Message" />
      <div data-pw="split-expression">Before {props.label}</div>
      <div data-pw={42}>Numeric selector</div>
      <div data-pw>Boolean selector</div>
      <div data-pw={`${props.label}`}>Dynamic template selector</div>
      <div svg:path="x" data-pw="namespaced-attr">Namespaced attr</div>
      <div data-pw=<span /> aria-label="Element attribute">Element attribute</div>
      <select {...props} data-pw="spread-select" multiple aria-label="Spread select" />
      <svg:path data-pw="namespaced-target">Namespaced</svg:path>
      <this.Button data-pw="this-button" title="This button">This button</this.Button>
      <foo.Button testId="lower-member-button" title="Lower member">Lower member</foo.Button>
      <a data-pw="undefined-link-target" href={undefined}>Undefined link</a>
      <a data-pw="null-link-target" href={null}>Null link</a>
      <div>{props.label}</div>
    </>
  );
}
