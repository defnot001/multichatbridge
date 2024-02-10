UPDATE users
SET auth_token = $1
WHERE server_id = $2
RETURNING server_id, server_list;