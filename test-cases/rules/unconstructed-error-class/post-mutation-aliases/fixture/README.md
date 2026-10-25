# Constructor aliases captured after mutation

The declaration describes a non-Error Pool class. A runtime replacement happens
before Snapshot captures that member, so neither local construction nor an
imported subclass can trust the original declaration identity. Both construction
sites must lack class proof. Their symbolic module targets retain the existing
quiet reporting policy; the imported subclass advises. The unrelated unused
Error must still fail. Do not move the mutation after the snapshot: that is a
different case.
