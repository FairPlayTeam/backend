SELECT id, password_hash
FROM public.users
WHERE email = $1;
