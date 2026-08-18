-- Nothing to undo: the up migration only filled in NULLs, and which rows
-- those were is not recorded. Clearing every icon would also discard the
-- ones `product_icon` set and any set by hand since.
SELECT 1;
