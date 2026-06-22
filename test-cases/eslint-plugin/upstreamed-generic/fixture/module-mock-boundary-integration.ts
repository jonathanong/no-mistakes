/* no-mistakes: integration=network */
export function allowed() {
  return "ok";
}

export function blocked() {
  return "no";
}

function localNamed() {
  return "named";
}

/* no-mistakes: integration=network */
export { localNamed as namedAllowed };

/* no-mistakes: integration=network */
export default function client() {
  return "default";
}
