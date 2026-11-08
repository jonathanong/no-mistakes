export default { test: { projects: [
  { test: { name: "owned", include: ["root.mts"], setupFiles: ["./setup.mts"] } },
  { test: { name: "other", include: ["other.mts"] } }
] } };
