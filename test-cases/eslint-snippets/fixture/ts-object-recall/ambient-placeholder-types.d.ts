declare module "@example/visible" {
  type Local = string;
  export type { Local };
}

declare module "@example/placeholder" {
  type Local = never;
  export type { Local };
}
