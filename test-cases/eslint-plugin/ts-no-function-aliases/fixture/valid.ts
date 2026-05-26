function transforms(value: string) {
  return original(value.trim());
}

function addsWork(value: string) {
  audit(value);
  return original(value);
}

const changesArguments = (value: string) => original(value, "extra");
const literal = () => 42;
api[method] = (value: string) => original(value);

export const isApiRoute = (pathname: string): boolean => API_ROUTE_RE.test(pathname);

export const isInfraRoute = (pathname: string): boolean => INFRA_ROUTE_RE.test(pathname);

export const isRssRoute = (pathname: string): boolean => RSS_ROUTE_RE.test(pathname);

export function isShellAssignment(token: string): boolean {
  return /^[A-Za-z_][A-Za-z0-9_]*=/.test(token);
}

export function registerGlideMQInstance(instance: { close(): Promise<void> }) {
  instances.push(instance);
}

export function clearArticleMarkdownCacheForTests(): void {
  markdownCache.clear();
}

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
