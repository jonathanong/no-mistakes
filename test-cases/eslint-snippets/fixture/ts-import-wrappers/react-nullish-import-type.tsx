declare const reactModule: string;

function B(x: import(reactModule).ReactNode) {
  return x ?? "fallback";
}
