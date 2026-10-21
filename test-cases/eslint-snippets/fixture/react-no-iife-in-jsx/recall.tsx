export function Shown() {
  return (
    <div>
      {(function show() {
        return 1;
      }).call(null)}
      {(function show() {
        return 1;
      }).apply(null, [])}
      {(0, function show() {
        return 1;
      })()}
      {(() => 1)?.()}
      {(function show() {
        return 1;
      }).bind(null)}
      {(function show() {
        return 1;
      })["call"](null)}
      {(function show() {
        return 1;
      }).bind(null)}
      {(function show() {
        return 1;
      })["call"](null)}
    </div>
  );
}

export function Hidden() {
  return (
    // no-mistakes-disable-next-line react-no-iife-in-jsx -- retained probe
    <span>{(() => 1)()}</span>
  );
}
