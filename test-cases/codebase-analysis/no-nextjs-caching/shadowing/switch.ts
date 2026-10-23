const mode = 'force-cache';
function switchScope() {
  // The discriminant is outside the shared lexical scope of all cases.
  switch (fetch('/discriminant', { cache: mode })) { // finding
    case fetch('/case-test', { cache: mode }):
      function earlier() { fetch('/before-case-binding', { cache: mode }); }
      const alias = mode;
      fetch('/alias-before-binding', { cache: alias });
      break;
    default:
      let mode = 'no-store';
      fetch('/case-body', { cache: mode });
  }
  fetch('/after-switch', { cache: mode }); // finding
}
function switchVar() {
  switch (value) { default: var mode = 'no-store'; }
  fetch('/after-switch-var', { cache: mode });
}
