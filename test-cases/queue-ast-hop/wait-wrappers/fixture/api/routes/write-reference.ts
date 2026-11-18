declare const client: any;
const submit = client.enqueueReviewAndWait;
// @ts-expect-error Fixture exercises a write reference on a const binding.
submit = () => {};
submit();
