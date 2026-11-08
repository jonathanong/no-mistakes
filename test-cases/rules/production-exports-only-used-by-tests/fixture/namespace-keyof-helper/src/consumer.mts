import * as templates from './templates.mjs';
function processEmail(t: typeof templates, name: keyof typeof templates) {
  return t[name]();
}
processEmail(templates, 'processX');
