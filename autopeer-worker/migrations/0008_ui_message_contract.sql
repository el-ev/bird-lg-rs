-- Rewrite persisted auth/session payloads into the structured UiMessage shape.
-- This is a one-time data migration for the Rust frontend's stricter decoding.

UPDATE auth_sessions
SET auth_method = CASE json_extract(auth_method, '$.kind')
  WHEN 'registry_ssh' THEN json_set(
    json_set(
      auth_method,
      '$.label',
      json_object('key', 'auth_method.registry_ssh.label')
    ),
    '$.description',
    json_object(
      'key',
      'auth_method.registry_ssh.session_description',
      'params',
      json_object('mnt', effective_mnt)
    )
  )
  WHEN 'registry_pgp' THEN json_set(
    json_set(
      auth_method,
      '$.label',
      json_object('key', 'auth_method.registry_pgp.label')
    ),
    '$.description',
    json_object(
      'key',
      'auth_method.registry_pgp.session_description',
      'params',
      json_object('mnt', effective_mnt)
    )
  )
  WHEN 'registry_email' THEN json_set(
    json_set(
      auth_method,
      '$.label',
      json_object('key', 'auth_method.registry_email.label')
    ),
    '$.description',
    json_object(
      'key',
      'auth_method.registry_email.session_description',
      'params',
      json_object('mnt', effective_mnt)
    )
  )
  WHEN 'host_impersonation' THEN json_set(
    json_set(
      auth_method,
      '$.label',
      json_object('key', 'auth_method.host_impersonation.label')
    ),
    '$.description',
    json_object(
      'key',
      'auth_method.host_impersonation.description',
      'params',
      json_object(
        'mnt',
        effective_mnt,
        'host_asn',
        CASE
          WHEN substr(json_extract(auth_method, '$.provider'), 1, 2) = 'AS' THEN substr(json_extract(auth_method, '$.provider'), 3)
          ELSE COALESCE(json_extract(auth_method, '$.provider'), '')
        END
      ),
      'fallback',
      COALESCE(json_extract(auth_method, '$.description'), '')
    )
  )
  WHEN 'oidc' THEN json_set(
    json_set(
      auth_method,
      '$.label',
      json_object(
        'key',
        COALESCE(json_extract(auth_method, '$.label'), json_extract(auth_method, '$.provider'), 'OIDC'),
        'fallback',
        COALESCE(json_extract(auth_method, '$.label'), json_extract(auth_method, '$.provider'), 'OIDC')
      )
    ),
    '$.description',
    json_object(
      'key',
      'auth_method.oidc.session_description',
      'params',
      json_object(
        'provider',
        COALESCE(json_extract(auth_method, '$.label'), json_extract(auth_method, '$.provider'), 'OIDC'),
        'mnt',
        effective_mnt
      ),
      'fallback',
      COALESCE(json_extract(auth_method, '$.description'), '')
    )
  )
  ELSE auth_method
END
WHERE json_type(auth_method, '$.label') = 'text'
   OR json_type(auth_method, '$.description') = 'text';

UPDATE operations
SET message = CASE
  WHEN message = 'We are preparing your pull request.' THEN
    json_object('key', 'operation.message.pending_pull_request')
  WHEN message = 'Your pull request is open; waiting for peer-session-check.' THEN
    json_object('key', 'operation.message.pending_checks')
  WHEN message = 'Checks passed; applying your session to the node for verification.' THEN
    json_object('key', 'operation.message.applying')
  WHEN message = 'Apply succeeded on the node; waiting for merge.' THEN
    json_object('key', 'operation.message.pending_merge')
  WHEN message = 'Your change was applied and merged successfully.' THEN
    json_object('key', 'operation.message.completed')
  WHEN message = 'Your change failed.' THEN
    json_object('key', 'operation.message.failed')
  WHEN message = 'We could not apply your change because our repo conflicted.' THEN
    json_object('key', 'operation.message.conflict')
  WHEN message = 'Apply succeeded; waiting for another change on this node to finish merging.' THEN
    json_object('key', 'operation.message.wait_node_lock')
  WHEN message = 'Your session already matches our repo, so we did not open a pull request.' THEN
    json_object('key', 'operation.message.no_change')
  WHEN message = 'peer-session-check did not start for your pull request.' THEN
    json_object('key', 'operation.message.check_not_started')
  WHEN message = 'Your pull request is open; waiting for peer-session-check to start.' THEN
    json_object('key', 'operation.message.check_wait_start')
  WHEN message = 'Checks passed; waiting for peer-session-apply to start.' THEN
    json_object('key', 'operation.message.apply_wait_start')
  WHEN message = 'peer-session-apply did not start for your pull request.' THEN
    json_object('key', 'operation.message.apply_not_started')
  WHEN message = 'Your pull request was closed before merge.' THEN
    json_object('key', 'operation.message.pull_request_closed')
  WHEN substr(
    message,
    1,
    length('peer-session-check finished with ')
  ) = 'peer-session-check finished with ' THEN
    json_object(
      'key',
      'operation.message.check_failed',
      'params',
      json_object(
        'conclusion',
        substr(message, length('peer-session-check finished with ') + 1)
      ),
      'fallback',
      message
    )
  WHEN substr(
    message,
    1,
    length('peer-session-apply finished with ')
  ) = 'peer-session-apply finished with ' THEN
    json_object(
      'key',
      'operation.message.apply_failed',
      'params',
      json_object(
        'conclusion',
        substr(message, length('peer-session-apply finished with ') + 1)
      ),
      'fallback',
      message
    )
  WHEN substr(
    message,
    1,
    length('Apply succeeded on the node; waiting for merge. Merge attempt failed: ')
  ) = 'Apply succeeded on the node; waiting for merge. Merge attempt failed: ' THEN
    json_object(
      'key',
      'operation.message.merge_failed',
      'params',
      json_object(
        'error',
        substr(
          message,
          length('Apply succeeded on the node; waiting for merge. Merge attempt failed: ') + 1
        )
      ),
      'fallback',
      message
    )
  ELSE
    json_object('key', message, 'fallback', message)
END
WHERE message IS NOT NULL
  AND json_valid(message) = 0;
