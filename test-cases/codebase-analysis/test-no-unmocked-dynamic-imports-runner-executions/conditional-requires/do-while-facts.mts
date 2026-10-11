// Built-in specifiers keep each condition distinct without unresolved imports.
do {} while (require('node:assert'));
do ; while (require('node:buffer'));
do { const value = 1; value; debugger; } while (require('node:constants'));
do { function nested() { return; } } while (require('node:crypto'));
do { if (globalThis.flag) {} } while (require('node:dns'));
do { if (globalThis.flag) {} else {} } while (require('node:domain'));
do { continue; } while (require('node:events'));
do { break; } while (require('node:fs'));
do { if (globalThis.flag) break; } while (require('node:http'));
do { if (globalThis.flag) {} else break; } while (require('node:https'));
do { throw new Error(); } while (require('node:module'));
outer: do { continue outer; } while (require('node:net'));
do { while (globalThis.flag) {} } while (require('node:os'));
function earlyReturn() { do { return; } while (require('node:path')); }
if (false) { do {} while (require('node:process')); }
