import { read, write } from '@vendor/plain-nullish';
// Unknown alternatives must retain independently known fallback effects.
export function run(candidate) {
  const query = candidate ?? read;
  query();
  (candidate ?? write)();
}
run(unresolved);
