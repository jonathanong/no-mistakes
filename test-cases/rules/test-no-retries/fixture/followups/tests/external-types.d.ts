export declare const test: {
  describe: {
    serial: (name: string, fn: () => void) => void;
    configure: (opts: { retries?: number; mode?: string }) => void;
  };
};
