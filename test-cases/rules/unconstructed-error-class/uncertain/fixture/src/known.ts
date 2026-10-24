export class NamedError extends Error {}
new NamedError();
export class ParentError extends Error {}
export class ChildError extends ParentError {}
new ChildError();
class Plain {}
export class NotAnError extends Plain {}

// Unrelated known dead classes remain definite under every uncertainty policy.
export class DefinitelyDead extends Error {}
