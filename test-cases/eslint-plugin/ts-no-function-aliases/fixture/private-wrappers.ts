export function noArgs() {
  return original();
}

export function withArg(value: string) {
  return original(value);
}

export function sideEffect(value: string) {
  original(value);
}

export const arrow = (value: string) => original(value);

export const asyncArrow = async (value: string) => await original(value);

export const restArrow = (...args: string[]) => original(...args);

exports.propertyArrow = (value: string) => original(value);

module.exports.propertyFunction = function (value: string) {
  return original(value);
};

api.property = (value: string) => original(value);

exports.sameName = (value: string) => sameName(value);

exports["computed"] = (value: string) => original(value);

// Meaningful exported names still count as aliases when their bodies only forward unchanged arguments.
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

async function asyncAlias(value: string) {
  return await original(value);
}

function outerAlias() {
  function nested(value: string) {
    return original(value);
  }
  return nested("x");
}

function original(value?: string) {
  return value;
}
