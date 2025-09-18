WITH new_user AS (
    INSERT INTO public.users (email, password_hash)
    VALUES ($2, $3)
    RETURNING id
)
INSERT INTO public.user_accounts (id, username)
SELECT id, $1
FROM new_user;
