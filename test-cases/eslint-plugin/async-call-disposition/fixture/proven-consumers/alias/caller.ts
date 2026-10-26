import { enqueueEmail } from "@app/jobs";
import { processReconciliationPage } from "@consumer";
const callback = () => enqueueEmail(1);
await processReconciliationPage(callback);
import { enqueueReferralLinkCrawlsForUrlId } from "@chain";
const forwarded = () => enqueueEmail(2);
await enqueueReferralLinkCrawlsForUrlId("url", { enqueueBulkCrawlReferralLinks: forwarded, computeHostnameRateLimitMs: async () => 1 });
