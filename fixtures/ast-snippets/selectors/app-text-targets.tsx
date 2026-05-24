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
      <label>Wrapped email <span><input data-pw="wrapped-email-input" /></span></label>
      <label><><input data-pw="fragment-email-input" /></>Fragment email</label>
      <span id="plan-label">Plan label</span>
      <input id="plan-input" data-pw="plan-input" aria-labelledby="plan-label" />
      <span id="button-label-no-id">No id label</span>
      <button data-pw="labelled-no-id" aria-labelledby="button-label-no-id" />
      <span id="override-label">Override name</span>
      <button data-pw="labelledby-precedence" aria-labelledby="override-label">Visible should not name</button>
      <button data-pw="dynamic-labelledby" aria-labelledby={props.label}>Dynamic visible should not name</button>
      <button data-pw="dynamic-aria-label" aria-label={props.label}>Dynamic aria visible should not name</button>
      <input id="missing-labelledby-input" data-pw="missing-labelledby-input" aria-labelledby="missing-label" />
      <label htmlFor="missing-control">Missing control</label>
      <input data-pw={"search-input"} aria-label={"Search field"} placeholder={"Search"} />
      <div data-pw="decorative-placeholder" placeholder="Decorative placeholder">Decorative placeholder</div>
      <img alt="Company logo" />
      <Button testId="save-button" title="Save changes">Save</Button>
      <div data-pw={props.label}>Dynamic selector</div>
      <div {...props}>Spread child</div>
      <div data-pw="string-child">{"String child"}</div>
      <div data-pw="joined-text">Hello {"World"}<span>ignored</span>{"Again"}</div>
      <UI.Button testId="member-button" title="Member save">Member</UI.Button>
      <Design.UI.Button testId="nested-member-button" title="Nested member">Nested</Design.UI.Button>
      <button id="html-id-button">HTML id</button>
      <button data-pw="numeric-text-button">{42}</button>
      <button data-pw="descendant-button"><span>Descendant save</span></button>
      <button data-pw="hidden-descendant-button"><span aria-hidden="true">Decorative hidden</span>Shown descendant</button>
      <button data-pw="combined-descendant-button"><span>Save</span> now</button>
      <button data-pw="split-descendant-button"><span>Before {props.label} After</span> done</button>
      <button data-pw="fragment-button"><>Fragment save</></button>
      <button data-pw="hidden-button" hidden>Hidden action</button>
      <button data-pw="hidden-string-button" hidden="false">String hidden action</button>
      <button data-pw="hidden-expression-string-button" hidden={"false"}>Expression hidden action</button>
      <button data-pw="hidden-false-button" hidden={false}>Shown action</button>
      <button data-pw="hidden-null-button" hidden={null}>Null shown action</button>
      <button data-pw="hidden-undefined-button" hidden={undefined}>Undefined shown action</button>
      <button data-pw="hidden-zero-button" hidden={0}>Zero shown action</button>
      <button data-pw="hidden-one-button" hidden={1}>One hidden action</button>
      <button data-pw="hidden-ts-button" hidden={true as const}>TS hidden action</button>
      <button data-pw="hidden-empty-template-button" hidden={``}>Empty template shown action</button>
      <button data-pw="hidden-template-button" hidden={`truthy`}>Template hidden action</button>
      <button data-pw="hidden-dynamic-button" hidden={props.label}>Dynamic shown action</button>
      <button data-pw="aria-hidden-button" aria-hidden="true">Aria hidden action</button>
      <button data-pw="aria-hidden-false-button" aria-hidden="false">Aria shown action</button>
      <button data-pw="aria-hidden-bool-button" aria-hidden={true}>Bool hidden action</button>
      <button data-pw="aria-hidden-expression-string-button" aria-hidden={"false"}>Expression string shown action</button>
      <button data-pw="aria-hidden-ts-button" aria-hidden={false as boolean}>TS aria shown action</button>
      <label>Wrapped email <input data-pw="wrapped-email-input" /></label>
      <label>Fragment wrapped <><input data-pw="fragment-wrapped-input" /></></label>
      <div data-pw="container-target"><button>Container child</button></div>
      <input data-pw="submit-input" type="submit" value="Submit form" />
      <input data-pw="submit-case-input" type="Submit" value="Case submit form" />
      <div data-pw="explicit-role" role="button">Explicit role</div>
      <div data-pw="fallback-role" role="unknown button">Fallback role</div>
      <div data-pw="deletion-role" role="deletion">Deleted text</div>
      <div data-pw="insertion-role" role="insertion">Inserted text</div>
      <button data-pw="aria-label-precedence" aria-label="Close">Visible close text</button>
      <button data-pw="title-only-button" title="Only title" />
      <button data-pw="button-alt" alt="Ignored alt" />
      <div data-pw="aria-label-div" role="button" aria-label="Div action" />
      <span id="action-first">First</span>
      <span id="action-second">Second</span>
      <div id="custom-action" data-pw="custom-action" role="button" aria-labelledby="action-first action-second" />
      <a data-pw="link-target" href="/docs">Docs</a>
      <a data-pw="expression-link-target" href={"/expression-docs"}>Expression docs</a>
      <a data-pw="template-link-target" href={`/template-docs`}>Template docs</a>
      <a data-pw="empty-link-target" href="">Empty docs</a>
      <a data-pw="dynamic-link-target" href={props.label}>Dynamic docs</a>
      <h2 data-pw="heading-target">Heading</h2>
      <img data-pw="image-target" alt="Hero image" />
      <input data-pw="image-input" type="image" alt="Image submit" />
      <input data-pw="checkbox-target" type="checkbox" aria-label="Subscribe" />
      <input data-pw="radio-target" type="radio" aria-label="Pick one" />
      <input data-pw="range-target" type="range" aria-label="Volume" />
      <input data-pw="searchbox-target" type="search" aria-label="Search site" />
      <input data-pw="number-target" type="number" aria-label="Count" />
      <input data-pw="hidden-target" type="hidden" aria-label="Hidden token" />
      <label htmlFor="hidden-labelled-input">Hidden label</label>
      <input id="hidden-labelled-input" data-pw="hidden-labelled-input" type="hidden" />
      <input data-pw="password-target" type="password" aria-label="Secret" />
      <select data-pw="select-target" aria-label="Country" />
      <select data-pw="listbox-target" multiple aria-label="Tags" />
      <select data-pw="multiple-false-target" multiple={false} aria-label="Single tag" />
      <select data-pw="sized-listbox-target" size="2" aria-label="Regions" />
      <select data-pw="numeric-sized-listbox-target" size={2} aria-label="Numeric regions" />
      <select data-pw="ts-sized-listbox-target" size={2 as const} aria-label="TS regions" />
      <textarea data-pw="textarea-target" aria-label="Message" />
      <div data-pw="split-expression">Before {props.label}</div>
      <div data-pw={42}>Numeric selector</div>
      <div data-pw>Boolean selector</div>
      <div data-pw={`${props.label}`}>Dynamic template selector</div>
      <div svg:path="x" data-pw="namespaced-attr">Namespaced attr</div>
      <div data-pw={<span />} aria-label="Element attribute">Element attribute</div>
      <select {...props} data-pw="spread-select" multiple aria-label="Spread select" />
      <svg:path data-pw="namespaced-target">Namespaced</svg:path>
      <this.Button data-pw="this-button" title="This button">This button</this.Button>
      <foo.Button testId="lower-member-button" title="Lower member">Lower member</foo.Button>
      <a data-pw="undefined-link-target" href={undefined}>Undefined link</a>
      <a data-pw="null-link-target" href={null}>Null link</a>
      <a data-pw="zero-link-target" href={0}>Zero link</a>
      <div>{props.label}</div>
    </>
  );
}
