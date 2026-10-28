declare function dynamic(loader: any): any;
declare function retry(loader: any): any;
// Used only in JSX, so the binding is never consumed and stays the default.
const Jsx = dynamic(() => import("./a"));
export const view = <Jsx />;
// A binding that is also read through a member is consumed, so it is unproven.
const Mixed = retry(() => import("./a"));
export const mixed = <Mixed />;
Mixed.then((m: any) => m.X);
// A member-style element name reads a property of the result, so it is consumed.
const Member = retry(() => import("./a"));
export const memberView = <Member.Inner />;
// JSX attributes and children are sinks, like any JSX expression container.
export const inAttribute = <div render={dynamic(() => import("./a"))}>{dynamic(() => import("./a"))}</div>;
// The `next/dynamic` shapes: a `.then` loader proves its member, a bare one proves `default`.
const Named = dynamic(() => import("./a").then((m: any) => m.Name));
const Plain = dynamic(() => import("./b"));
export const nextViews = [<Named />, <Plain />];
