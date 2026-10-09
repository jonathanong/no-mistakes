-- Index expressions stay outside configured clauses; only predicates are WHERE.
CREATE INDEX partial_events ON events (probe_index_expression(id))
WHERE probe_partial_where(probe_predicate_arg(id)) > 0;
CREATE INDEX all_events ON events (probe_plain_index(id));
