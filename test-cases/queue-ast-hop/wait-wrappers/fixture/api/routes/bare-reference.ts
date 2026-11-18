declare function enqueueReviewAndWait(): void;
declare function registerCallback(callback: () => void): void;
registerCallback(enqueueReviewAndWait);
