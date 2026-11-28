declare const reactModule: string;
const R = require(reactModule);
interface Props {
  child: R.ReactNode;
}
export function named(props: Props) {
  return props.child ?? "fallback";
}
export function aliased(props: Props) {
  const alias = props;
  return alias.child ?? "fallback";
}
export function shadowed(props: Props) {
  return function inner(p: { child: string }) {
    return p.child ?? "";
  };
}
