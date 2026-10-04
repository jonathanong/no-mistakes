-- BigQuery omits the FROM keyword in DELETE; exercise sqlparser's alternate AST variant.
DELETE accounts WHERE FALSE;
