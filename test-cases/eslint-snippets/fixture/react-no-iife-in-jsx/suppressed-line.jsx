export function Shown() {
  return <span>{(() => 1)()}</span>; // no-mistakes-disable-line react-no-iife-in-jsx -- retained probe
}
