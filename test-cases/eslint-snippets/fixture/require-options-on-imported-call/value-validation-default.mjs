import send from '@example/transport'
import agents from '@example/agents'
await send(request, { agent: agents.otherAgent() })
await send(request, { agent: agents.approvedAgent() })
await send(request, { agent }, extra)
await send(request, { agent })
