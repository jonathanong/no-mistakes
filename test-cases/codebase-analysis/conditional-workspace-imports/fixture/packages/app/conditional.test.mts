// The callback is intentionally not proven to execute.
declare function it(name: string, callback: () => Promise<void>): void;
it("conditional", async () => {
  await Promise.all([
    import("./relative.mts"),
    import("@fx/lib/target-a"),
    import("@fx/lib/target-b"),
    import("#local/target"),
  ]);
});
