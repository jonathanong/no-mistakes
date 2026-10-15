These are deliberately incomplete Cargo packages for conservative planner
handling: missing declared paths, malformed manifests, missing package names,
and unnamed target declarations must not borrow another package's ownership.
No test executes the generated Cargo commands for these malformed packages.
