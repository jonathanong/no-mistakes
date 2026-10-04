-- Quoted access methods are distinct names; unquoted names fold case.
CREATE INDEX quoted_first ON accounts USING "CustomAM" (id);
CREATE INDEX quoted_second ON accounts USING "CUSTOMAM" (id);
CREATE INDEX unquoted_first ON accounts USING CustomAM (id);
CREATE INDEX unquoted_second ON accounts USING customam (id);
