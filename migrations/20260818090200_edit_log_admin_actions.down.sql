DELETE FROM edit_log WHERE action IN ('approve', 'reject', 'restore');
ALTER TABLE edit_log DROP CONSTRAINT edit_log_action_check;
ALTER TABLE edit_log ADD CONSTRAINT edit_log_action_check
    CHECK (action IN ('update', 'delete'));
