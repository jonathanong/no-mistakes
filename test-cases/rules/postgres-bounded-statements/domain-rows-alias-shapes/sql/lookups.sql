SELECT owner_id FROM seats AS renamed(label) WHERE owner_id = $1;
SELECT owner_id FROM seats AS renamed(label, owner_id, band) WHERE owner_id = $1;
SELECT owner_id FROM shadows AS renamed(slot, band) WHERE slot = $1;
SELECT owner_id FROM gaps AS renamed(note) WHERE owner_id = $1;
SELECT owner_id FROM others WHERE owner_id = $1;
SELECT owner_id FROM audit.bands WHERE owner_id = $1;
SELECT owner_id FROM audit.misses WHERE owner_id = $1;
