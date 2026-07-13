import { ignoredFetch } from "./ignored-helper";
import { ignoredBridge } from "./ignored-bridge";

ignoredFetch();
ignoredBridge();
fetch("/api/visible-page");
