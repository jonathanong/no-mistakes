declare function register(callback: () => unknown): void;
import("./src/scan.mts").then((module) => register(module.runSmoke));
