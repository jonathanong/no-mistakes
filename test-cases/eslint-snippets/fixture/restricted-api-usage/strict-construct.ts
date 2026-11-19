new RateLimitedWorker();
new (api.RateLimitedWorker as any)();
new RateLimitedWorker(...rest);
// Controls: arity differs, other names, calls and computed callees.
new RateLimitedWorker(options);
new OtherWorker();
RateLimitedWorker();
new (factory())();
