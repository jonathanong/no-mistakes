import "./unreachable";
// Overlapping roots share a cyclic tail: policies must visit it only once.
export function alpha() { shared(); }
export function beta() { shared(); }
export function shared() { tail(); }
export function tail() { shared(); leaf(); }
export function leaf() { setTimeout(() => {}, 0); }
export function isolated() { setInterval(() => {}, 0); }
export { alpha as aliasAlpha };
alpha();
