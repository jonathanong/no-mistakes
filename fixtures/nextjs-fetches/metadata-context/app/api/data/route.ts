export async function GET() {
  const data = await fetch('/api/external');
  return Response.json(data);
}
