declare function dynamic(loader: any): any;
declare function retry(loader: any): any;
// Used only in JSX, so the binding is never consumed and stays the default.
const Jsx = dynamic(() => import("./a"));
export const view = <Jsx />;
// A binding that is also read through a member is consumed, so it is unproven.
const Mixed = retry(() => import("./a"));
export const mixed = <Mixed />;
Mixed.then((m: any) => m.X);
