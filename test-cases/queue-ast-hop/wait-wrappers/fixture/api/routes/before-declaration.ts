declare const client: any;
// @ts-expect-error Fixture exercises a reference before its declaration.
submit();
const { enqueueReviewAndWait: submit } = client;
