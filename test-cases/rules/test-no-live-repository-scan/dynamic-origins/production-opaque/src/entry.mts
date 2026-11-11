declare const modulePath: string;
if (process.env.RUN_SMOKE === "1") {
  await import(modulePath);
}
