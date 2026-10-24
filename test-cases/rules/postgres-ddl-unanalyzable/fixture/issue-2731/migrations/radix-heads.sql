-- Rewritten recursive-view heads must retain the authored radix literals.
CREATE RECURSIVE VIEW hex_head AS SELECT 0x1F;
CREATE RECURSIVE VIEW binary_head AS SELECT 0b101;
FROBULATE 0x1F;
