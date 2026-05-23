// guardrails-disable-file nextjs-no-api-routes
export async function GET() {
  return Response.json({ ignored: true })
}
