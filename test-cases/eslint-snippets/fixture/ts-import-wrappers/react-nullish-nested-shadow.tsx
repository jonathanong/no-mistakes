declare const moduleName: string;
const R = require(moduleName);

interface Props {
  child: R.ReactNode;
}

export function outer(props: Props) {
  return props.child ?? "";
}

export function nested() {
  type Props = { child: string };
  function inner(props: Props) {
    return props.child ?? "";
  }
  return inner;
}

export function caught(value: R.ReactNode) {
  try {
    return value ?? null;
  } catch (value) {
    return value ?? null;
  }
}

export function blockFunction(value: R.ReactNode) {
  {
    function value() {
      return value ?? null;
    }
    return value;
  }
}
