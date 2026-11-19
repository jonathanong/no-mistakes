import * as transport from '@example/transport'
import * as agents from '@example/agents'
await transport.send(request, { agent: agents.otherAgent() })
await transport.send(request, { agent: agents.approvedAgent() })
await transport.send(request, { agent }, extra)
await transport.send(request, { agent })
