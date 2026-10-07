interface Foo {}
type Alias = string;
import { type Shape } from "./external";
const value = 1;
export { Foo, Alias, Shape, value };
export { Shape as Forwarded } from "./external";
