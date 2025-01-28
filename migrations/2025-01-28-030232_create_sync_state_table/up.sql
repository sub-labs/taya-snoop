CREATE TABLE sync_state (
  id TEXT PRIMARY KEY NOT NULL,
  last_block_number BIGINT NOT NULL
);

INSERT INTO sync_state (id, last_block_number) VALUES ('sync_state', 0);
