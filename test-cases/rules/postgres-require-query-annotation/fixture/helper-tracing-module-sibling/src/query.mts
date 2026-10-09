import { mutate, send } from './state.mjs';
// Sibling arms never observe mutations from the other arm.
const selected = flag ? mutate() : send();
