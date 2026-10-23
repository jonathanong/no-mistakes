const original = value => value;
export default original;
export const facade = value => original(value);

function liveOriginal(value) { return value; }
export { liveOriginal };
liveOriginal = replacement;
export const liveFacade = value => liveOriginal(value);
