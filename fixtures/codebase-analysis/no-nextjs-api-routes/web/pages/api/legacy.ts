export default function handler(_req: unknown, res: { json(value: unknown): void }) {
  res.json({ ok: true })
}
