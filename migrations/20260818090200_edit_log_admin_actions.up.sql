-- `edit_log` only accepted 'update' and 'delete', so the admin actions
-- the moderation UI performs had nowhere to be recorded — there was no
-- way to answer "who approved this". Approvals and restores carry a
-- snapshot in `old_value` like every other entry; `new_value` stays null
-- for reject/restore, which change one flag rather than a row's contents.
ALTER TABLE edit_log DROP CONSTRAINT edit_log_action_check;
ALTER TABLE edit_log ADD CONSTRAINT edit_log_action_check
    CHECK (action IN ('update', 'delete', 'approve', 'reject', 'restore'));
