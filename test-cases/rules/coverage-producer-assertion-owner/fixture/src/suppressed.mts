import { execute } from './production.mjs';
// no-mistakes-disable-next-line coverage-producer-assertion-owner -- deliberate coverage-only fixture
export function lineSuppressed() { execute(); }
export function lineInlineSuppressed() { execute(); } // no-mistakes-disable-line coverage-producer-assertion-owner
