INSERT INTO configs (client_id, subscriptions)
VALUES ($1, $2)
ON CONFLICT(client_id) DO UPDATE SET subscriptions = $2
RETURNING *;