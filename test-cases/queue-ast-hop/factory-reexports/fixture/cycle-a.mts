// The two files only re-export each other. Lookup must stop, and the failed hop
// must stay visible as an unresolved producer.
export { q3 } from "./cycle-b.mts";
