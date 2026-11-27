// no-mistakes-disable-file react-no-iife-in-jsx -- retained probe
export function Shown() {
  return <span>{(() => 1)()}</span>;
}
