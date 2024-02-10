UPDATE users
SET server_list = $1,
    auth_token  = $2
WHERE server_id = $3
RETURNING server_id, server_list