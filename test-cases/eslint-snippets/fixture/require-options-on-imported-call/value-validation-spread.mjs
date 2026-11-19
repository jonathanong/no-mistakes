import { send } from '@example/transport'
import { approvedAgent } from '@example/agents'
await send(request, { agent: approvedAgent() }, ...args)
await send(...args, { agent: approvedAgent() })
await send(request, { agent: approvedAgent() })
