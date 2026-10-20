// These names are metadata, text, and a switch on another value, never job dispatch.
export const metadata = { name: 'metadata' };
export const example = "job.name === 'text'";
export function process(job: { data: string }) {
  switch (job.data) { case 'payload': return { name: 'result' }; }
}
