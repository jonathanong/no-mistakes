// This duplicate has the same resolved origin as identity-a.ts. The directive
// must not turn suppression metadata into a distinct origin identity.
// no-mistakes-disable-next-line unique-exports: identity compatibility barrel
export { identity } from '../shared/identity-origin';
