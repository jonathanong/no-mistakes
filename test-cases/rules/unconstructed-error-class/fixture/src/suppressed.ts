// no-mistakes-disable-next-line unconstructed-error-class: thrown by the platform
export class SuppressedError extends Error {}

// The directive on the same line is honored too.
export class LineSuppressedError extends Error {} // no-mistakes-disable-line unconstructed-error-class: thrown by the platform
