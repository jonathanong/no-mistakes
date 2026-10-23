async function reads() {
  let absent;
  ordinary();
  await ordinary();
  await client.get('a'), ordinary();
  const a = await client.get('a'), b = 2;
  await client.get('a');
  await client.get('b');
  { await client.get('c'); await client.get('d'); }
}
async function otherShapes() {
  await pending;
  await holder.client.get('a');
}
