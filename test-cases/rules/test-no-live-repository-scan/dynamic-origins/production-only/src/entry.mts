if (process.env.RUN_SMOKE === "1") {
  const { runSmoke } = await import("./scan.mts");
  runSmoke();
}
