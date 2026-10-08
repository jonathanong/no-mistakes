// The callback is intentionally not proven to execute.
declare function it(name: string, callback: () => Promise<void>): void;
it("conditional", async () => {
  await import("./relative.mts");
  await import("@fx/lib/target-a");
  await import("@fx/lib/target-b");
  await import("#local/target");
});
