export function command() { commandEffect(); }
function plain() { plainEffect(); }
function strict() { strictEffect(); }
function conjunction() { conjunctionEffect(); }
function alternate() { alternateEffect(); }
function negated() { negatedEffect(); }
function namedCallback() { namedCallbackEffect(); }
function after() { afterEffect(); }
if (import.meta.main) { plain(); command(); (() => callbackEffect())(); consume(namedCallback); }
else { alternate(); }
if (import.meta.main === true) { strict(); }
if (true === import.meta.main) { strict(); }
if (import.meta.main && enabled) { conjunction(); }
if (enabled && import.meta.main) { conjunction(); }
if (!import.meta.main) { negated(); }
after();
if (import.meta.main) {
  if (import.meta.main) { plain(); }
  strict();
}
