const mode = 'force-cache';
function loops() {
  for (const mode of (fetch('/of-rhs', { cache: mode }), items)) {
    fetch('/of-body', { cache: mode });
  }
  fetch('/after-of', { cache: mode }); // finding
  for (let mode in (fetch('/in-rhs', { cache: mode }), items)) {
    fetch('/in-body', { cache: mode });
  }
  fetch('/after-in', { cache: mode }); // finding
  for (let mode = fetch('/init', { cache: mode }); fetch('/test', { cache: mode }); fetch('/update', { cache: mode })) {
    fetch('/body', { cache: mode });
  }
  fetch('/after-for', { cache: mode }); // finding
  for (;;) { fetch('/empty-head', { cache: mode }); break; } // finding
  for (mode of items) { fetch('/assignment-head', { cache: mode }); } // finding
}
function forVar() {
  for (var mode = 0; condition;) {}
  fetch('/after-var-for', { cache: mode });
}
function forInVar() {
  for (var mode in items) {}
  fetch('/after-var-in', { cache: mode });
}
