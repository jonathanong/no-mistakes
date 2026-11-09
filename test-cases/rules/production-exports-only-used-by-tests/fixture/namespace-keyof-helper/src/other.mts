import * as templates from './templates.mjs';
function invoke(t: typeof templates, name: keyof typeof templates) {
  return t[name]();
}
invoke(templates, 'processY');
