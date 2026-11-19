import { send } from '@example/transport'
import { approvedAgent, otherAgent } from '@example/agents'
await send(request, { agent: otherAgent() })
await send(request, { agent: approvedAgent() })
await send(request, { agent }, extra)
await send(request, { agent })
