-- An explicit alias named like the table hides its schema-qualified name; the lateral reference
-- therefore reaches the unaliased outer relation, while the middle scan remains email-pinned.
SELECT 1
FROM public.accounts
CROSS JOIN LATERAL (
    SELECT accounts.id
    FROM public.accounts accounts
    CROSS JOIN LATERAL (
        SELECT public.accounts.id
        LIMIT 1
    ) AS nested_reference
    WHERE accounts.email = $1
) AS bounded_middle;
