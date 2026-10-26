import { enqueueEmail } from "@app/jobs";
import { processReconciliationPage, processReferrals, unknownConsumer } from "./cross-file-consumers";
const enqueueContinuation = () => enqueueEmail(1);
await processReconciliationPage(enqueueContinuation);
const enqueueBulkCrawlReferralLinks = () => enqueueEmail(2);
await processReferrals({ enqueueBulkCrawlReferralLinks });
const unproven = () => enqueueEmail(3);
unknownConsumer(unproven);
