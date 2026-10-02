SELECT (SELECT COUNT(nullable_value) FROM entries) > 0;
SELECT (SELECT COUNT(nullable_value) FROM entries) = 0;
SELECT COUNT(nullable_value) > 0 FROM entries;
