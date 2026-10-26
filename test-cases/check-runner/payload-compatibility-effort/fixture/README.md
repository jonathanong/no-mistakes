# Declared payload compatibility effort matrix

`default.yml` and `max.yml` produce the same three findings: the two flat
incompatibilities plus `nested-incompatible`. The nested compatible pair and
the scalar, flat-object, root-array, and pointer controls pass at max.

At low report, the two flat incompatibilities remain definite and each nested
contract produces one contract-level `unproven` finding, for four findings.
At low advise, the two definite findings remain and the two nested contracts
are advisories. At low ignore, only the two definite findings remain. `mixed.yml`
uses disjoint contracts to check one low advisory plus one low definite finding
alongside two independent max findings.

`custom-message.yml` applies the same low-advise contract set with a custom
message prefix; the incompatible and unproven classifications remain in each
finding/advisory.

The main documents use RFC 6901 pointers into `$defs`. `wrapperdefs.json` also
checks one supported scalar through a pointer while leaving an unused `$ref`
outside the selected schema. `$comment` values explain fixture intent; there
are no JSON comments or inline-generated documents.
