export async function extend(extra: unknown) {
  const loaded = await import("./dynamicMemberControls");
  loaded.DYNAMIC_AWAIT.append(extra);
  await import("./dynamicMemberControls").then((parts) => {
    parts.DYNAMIC_THEN.append(extra);
  });
}
