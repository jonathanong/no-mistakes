const props = {};
const destination = "/items";

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
export const expression = <a href="/items">{<button type="button">Save</button>}</a>;
export const spreadChild = <button type="button">{...items}</button>;
export const commentChild = <button type="button">{/* comment */}Save</button>;
export const siblings = <div><a href="/items">Items</a><button type="button">Save</button></div>;
export const plainChild = <button type="button"><span>Save</span></button>;
