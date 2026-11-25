const props = {};
const destination = "/items";
const label = "Save";
const translate = (key) => key;

export const buttonInButton = (
  <button type="button">
    <>
      <button type="button" disabled>Save</button>
    </>
  </button>
);

export const linkInButton = (
  <button type="button">
    <a href={"/items"}>Items</a>
  </button>
);

export const linkInLink = (
  <a {...props} href="/items">
    <strong><a href="">Items</a></strong>
  </a>
);

export const placeholder = <a><button type="button">Save</button></a>;
export const dynamic = <a href={destination}><button type="button">Save</button></a>;
export const dynamicInner = <button type="button"><a href={destination}>Items</a></button>;
export const dynamicPlain = <a href={destination}><span>Items</span></a>;
export const overwritten = <a href="/items" {...props}><button type="button">Save</button></a>;
export const spreadSelfClosing = <button type="button" {...props} />;
export const cleared = <a href="/items" href={null}><button type="button">Save</button></a>;
export const valueLess = <a href><button type="button">Save</button></a>;
export const otherAttribute = <a title="Items"><button type="button">Save</button></a>;
export const namespaced = <svg><a href="/items"><button type="button">Save</button></a></svg>;
export const component = <a href="/items"><Wrapper><button type="button">Save</button></Wrapper></a>;
export const paragraphWrapper = <a href="/items"><p><button type="button">Save</button></p></a>;
export const memberWrapper = <button type="button"><UI.Icon /></button>;
export const expressionElement = <a href="/items">{<button type="button">Save</button>}</a>;
export const labelExpression = <button type="button">{label}</button>;
export const callExpression = <button type="button">{translate("save")}</button>;
export const iconChild = <a href="/items"><Icon /></a>;
export const stringLiteralChild = <button type="button">{"Save"}</button>;
export const templateLiteralChild = <button type="button">{`Save ${label}`}</button>;
export const parenthesizedLiteralChild = <button type="button">{("Save")}</button>;
export const childrenPropNested = <button type="button" children={<a href="/items">Items</a>} />;
export const childrenPropFragment = <button type="button" children={<><a href="/items">Items</a></>} />;
export const wrapperChildrenPropNested = <button type="button"><span children={<a href="/items">Items</a>} /></button>;
export const childrenPropExpression = <button type="button" children={label} />;
export const childrenPropLiteral = <button type="button" children="Save" />;
export const wrapperSpread = <button type="button"><span {...props} /></button>;
export const wrapperSpreadAfterChildren = <button type="button"><span children="Save" {...props} /></button>;
export const wrapperSpreadBeforeChildren = <button type="button"><span {...props} children="Save" /></button>;
export const wrapperSpreadWithJsxChild = <button type="button"><span {...props}>Save</span></button>;
export const callbackAttribute = <button type="button" onClick={() => <a href="/items">Items</a>}>Save</button>;
export const spreadChild = <button type="button">{...items}</button>;
export const commentChild = <button type="button">{/* comment */}Save</button>;
export const siblings = <div><a href="/items">Items</a><button type="button">Save</button></div>;
export const plainChild = <button type="button"><span>Save</span></button>;
