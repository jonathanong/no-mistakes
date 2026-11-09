import * as templates from './templates.mjs'; // no-mistakes-disable-line production-exports-only-used-by-tests
function processEmail(t: typeof templates, name: keyof typeof templates) {
  return t[name]();
}
processEmail(templates, 'processX');
