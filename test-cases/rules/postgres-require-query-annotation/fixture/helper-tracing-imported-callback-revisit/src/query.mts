import { send } from "./helper.mjs";
const callback = () => send();
const consumed = opaque(callback);
