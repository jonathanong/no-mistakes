// These names are metadata, text, and a switch on another value, never job dispatch.
export const metadata = { name: 'metadata' };
export const example = "job.name === 'text'";
export function process(job: { data: string }) {
  const items = [{ name: 'item' }];
  items.map((item) => {
    // A nested callback parameter is not the processor's active job.
    if (item.name === 'nested') return;
  });
  switch (job.data) { case 'payload': return { name: 'result' }; }
}
