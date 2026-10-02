UPDATE orders SET UPDATED_AT = now() WHERE id = $1;
