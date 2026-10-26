const mode = 'force-cache';
function caught() {
  try {} catch (mode) {
    fetch('/caught', { cache: mode });
    function earlier() { fetch('/nested-caught', { cache: mode }); }
  }
  fetch('/after-catch', { cache: mode }); // finding
  try {} catch ({ mode, ...rest }) {
    fetch('/destructured', { cache: mode });
    fetch('/rest', { cache: rest });
  }
  try {} catch {
    fetch('/without-binding', { cache: mode }); // finding
  }
}
