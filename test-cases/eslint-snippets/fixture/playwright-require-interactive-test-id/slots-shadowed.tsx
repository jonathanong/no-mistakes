import { Button as Action } from "./slot-components";
const Alias = Action;
function wrapper(Action) {
  const Alias = Action;
  return <Alias asChild data-pw="shadowed"><a href="/shadowed" /></Alias>;
}
function direct(Action) {
  return <Action asChild data-pw="shadowed"><a href="/direct" /></Action>;
}
