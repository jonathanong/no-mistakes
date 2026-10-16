/** Options for the configured `declared-payload-compatibility` check rule. */
export interface DeclaredPayloadCompatibilityOptions {
  contracts?: DeclaredPayloadContract[];
}

/** Producer declarations must accept only values accepted by the consumer. */
export interface DeclaredPayloadContract {
  name: string;
  producer: DeclaredPayloadSchema;
  consumer: DeclaredPayloadSchema;
}

/** A visible JSON document selected explicitly; no runtime adoption is inferred. */
export interface DeclaredPayloadSchema {
  /** Root-relative path within this rule application's visible file scope. */
  file: string;
  /** RFC 6901 JSON pointer; omitted or empty selects the whole document. */
  pointer?: string;
}
