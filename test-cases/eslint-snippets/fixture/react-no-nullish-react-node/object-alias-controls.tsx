import type { ReactNode } from "react";
type Props = { children: ReactNode };

export function dynamic(props: Props, children: string) {
  // The identifier key is a runtime name, even when it spells a known prop.
  const { [children]: slot } = props;
  return slot ?? null;
}

export function shadowed(props: Props) {
  function inner(props: { children: string | undefined }) {
    const p = props;
    return p.children ?? null;
  }
  return inner;
}

export function mutableSource(props: Props) {
  const p = props;
  props = { children: "changed" };
  return p.children ?? null;
}

export function mutableAlias(props: Props) {
  let p = props;
  p = { children: "changed" };
  return p.children ?? null;
}

export function temporal(props: Props) {
  {
    const p = props;
    const props = { children: "local" };
    return p.children ?? null;
  }
}

export function runtime(props: Props, choose: (props: Props) => Props) {
  const p = choose(props);
  return p.children ?? null;
}

export function explicitContract(props: Props) {
  const p: { children: string | undefined } = props;
  return p.children ?? null;
}

export function safe(props: { children: string | undefined }) {
  const { [`children`]: slot } = props;
  const p = props;
  return [slot ?? null, p.children ?? null];
}

export function defaulted(props: Props, p = props) {
  return p.children ?? null;
}

export function missing() {
  const p = unknownProps;
  return p.children ?? null;
}

export function reboundTarget(props: Props) {
  // Invalid const writes still erase the proof instead of retaining stale facts.
  const p = props;
  p = { children: "changed" };
  return p.children ?? null;
}

export function mutableDeclaration() {
  let props: Props = { children: "label" };
  const p = props;
  return p.children ?? null;
}

export function multiplyDeclared() {
  var props: Props;
  var props: Props;
  const p = props;
  return p.children ?? null;
}

export function functionShadow(props: Props) {
  {
    const p = props;
    function props() {}
    return p.children ?? null;
  }
}

export function parameterInitializer(props: Props, value = (() => {
  // Initializers can run before later parameters are initialized.
  const p = props;
  return p.children ?? null;
})()) {
  return value;
}

export function dynamicTemplate(props: Props) {
  const { [`children${""}`]: slot } = props;
  return slot ?? null;
}
