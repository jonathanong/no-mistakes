const origins = ["https://api.example.com", "https://cdn.example.com"] as const;

export { origins as registeredOrigins } from "./reexport-other";
