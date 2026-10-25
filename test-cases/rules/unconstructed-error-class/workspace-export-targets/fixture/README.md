# Workspace export construction proof

This fixture uses pnpm workspace packages and package export conditions. Root package metadata does not declare npm workspaces; pnpm-workspace.yaml owns
the workspace list. There are no tsconfig paths and no node_modules links. The app imports a package root
whose multiple star exports lead through named subpath re-exports into another
workspace package. Constructor definitions are outside the rule include, while
app Error subclasses protect exact construction identity and unused-error recall.
