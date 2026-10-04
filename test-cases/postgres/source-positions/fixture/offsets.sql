-- Unicode before each keyword checks character columns, not byte columns.
SELECT 'é' OFFSET /* nested /* inner */ comment */ 0;
SELECT 'λ' OFFSET
  2;
SELECT U&'\0061' OFFSET 3;
