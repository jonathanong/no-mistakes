// Re-export barrel. It references `used` (so `used` is not dead) but never
// calls it, so call-sites must not report a site here.
export { used } from "./util";
