-- Invalid labels before IF belong to the failed opener, not its child action.
<<condition>> IF true THEN
CREATE RECURSIVE VIEW invalid_label AS SELECT 0x1F;
-- Labels on loops are valid and the failed SQL action owns its own location.
<<body>> LOOP
CREATE RECURSIVE VIEW valid_label AS SELECT 0b101;
