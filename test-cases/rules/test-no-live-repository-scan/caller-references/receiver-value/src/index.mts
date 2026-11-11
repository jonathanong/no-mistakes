export { scan as zipDir } from "./scan.mts";
export function other() { return "other"; }
export const version = "1";
declare function makeHandler(): () => void;
export const handler = makeHandler();
