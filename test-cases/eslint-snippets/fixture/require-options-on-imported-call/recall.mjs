import guard from "ssrf-guard/node";
import { validateUrl } from "ssrf-guard/node";

guard.validateUrl(url);
const check = validateUrl;
check(url);
validateUrl.call(null, url);
(0, validateUrl)(url);
guard[`validateUrl`](url);
guard?.validateUrl(url);
