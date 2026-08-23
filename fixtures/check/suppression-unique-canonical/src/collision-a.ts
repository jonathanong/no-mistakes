// Lexically first barrel is suppressed, but must not hide the unsuppressed
// same-origin barrel or the distinct collision below.
// no-mistakes-disable-next-line unique-exports: compatibility barrel
export { collision } from '../shared/collision-origin';
