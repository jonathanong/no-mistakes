export function Suppressed() {
  // no-mistakes-disable-next-line csp-allowed-origins
  void fetch("https://next-line-suppressed.example.net/a");
  void fetch("https://same-line-suppressed.example.net/b"); // no-mistakes-disable-line csp-allowed-origins
  return null;
}
