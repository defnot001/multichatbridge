INSERT INTO configs (identifier, server_id, client_id, subscriptions)
VALUES ($1, $2, $3, $4)
ON CONFLICT(identifier) DO UPDATE SET subscriptions = $4
RETURNING *;