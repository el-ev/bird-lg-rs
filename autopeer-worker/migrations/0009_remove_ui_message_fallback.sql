-- Drop deprecated UiMessage fallback payloads now that v1 treats `key` as the sole source.

UPDATE auth_sessions
SET auth_method = json_remove(
  json_remove(auth_method, '$.label.fallback'),
  '$.description.fallback'
)
WHERE json_valid(auth_method) = 1
  AND (
    json_type(auth_method, '$.label.fallback') IS NOT NULL
    OR json_type(auth_method, '$.description.fallback') IS NOT NULL
  );

UPDATE auth_challenges
SET method_snapshot = (
  SELECT json_group_array(
    json_remove(
      json_remove(value, '$.label.fallback'),
      '$.description.fallback'
    )
  )
  FROM json_each(auth_challenges.method_snapshot)
)
WHERE json_valid(method_snapshot) = 1
  AND EXISTS (
    SELECT 1
    FROM json_each(auth_challenges.method_snapshot)
    WHERE json_type(value, '$.label.fallback') IS NOT NULL
       OR json_type(value, '$.description.fallback') IS NOT NULL
  );

UPDATE operations
SET message = json_remove(message, '$.fallback')
WHERE message IS NOT NULL
  AND json_valid(message) = 1
  AND json_type(message, '$.fallback') IS NOT NULL;
