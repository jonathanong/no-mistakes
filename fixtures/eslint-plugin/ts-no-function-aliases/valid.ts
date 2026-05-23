function transforms(value: string) {
  return original(value.trim());
}

function addsWork(value: string) {
  audit(value);
  return original(value);
}

const changesArguments = (value: string) => original(value, "extra");
const literal = () => 42;

function original(value: string, extra?: string) {
  return extra ? value + extra : value;
}

function audit(_value: string) {}

if (!candidates.some((candidate) => trackedFileSet.has(candidate))) {
  audit("missing");
}

await waitFor(() => {
  expect(screen.getByText("2")).toBeInTheDocument();
});

afterEach(() => {
  vi.useRealTimers();
});

proc.stdout.on("data", (chunk: Buffer) => chunks.push(chunk));
