// Its default export is only "seen" by a star barrel, which does not forward
// defaults — so it must be reported dead.
export default function lonelyDefault() {
  return 0;
}
