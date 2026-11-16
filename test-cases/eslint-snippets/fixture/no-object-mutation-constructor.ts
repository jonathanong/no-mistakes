export class Failure extends Error {
  readonly reason: unknown;
  items: unknown[];
  constructor(reason: unknown) {
    super("failure");
    this.reason = reason;
    this.items = [];
  }
}
export class LiteralFields {
  "value": unknown;
  ["other"]: unknown;
  constructor(value: unknown) {
    if (value) this["value"] = value;
    this.other = value;
  }
}
