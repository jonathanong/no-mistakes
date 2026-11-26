// no-mistakes-disable-file csp-allowed-origins
export function SuppressedFile() {
  void fetch("https://file-suppressed.example.net/events");
}
