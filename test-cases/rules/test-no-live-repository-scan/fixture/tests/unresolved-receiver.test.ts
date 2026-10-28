// The receiver is a parameter, so the call cannot be tied to `fs`; it is still reported.
export function scan(client: { readdirSync(path: string): string[] }) {
  return client.readdirSync(".");
}

scan(undefined as never);
