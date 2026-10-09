import { mutate, send } from './state.mjs';
// Both source orders must use the same pristine lazy-module input.
const selected = flag ? send() : mutate();
