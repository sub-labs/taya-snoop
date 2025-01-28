CREATE TABLE sync_state (
  id TEXT PRIMARY KEY NOT NULL,
  last_block_number BIGINT NOT NULL
);

INSERT INTO sync_state (id, last_block_number) VALUES ('sync_state', 0);

CREATE TABLE logs (
  address TEXT NOT NULL,
  block_number BIGINT NOT NULL,
  block_hash TEXT NOT NULL,
  chain BIGINT NOT NULL,
  data TEXT NOT NULL,
  log_index BIGINT NOT NULL,
  removed BOOLEAN NOT NULL,
  timestamp BIGINT NOT NULL,
  topic0 TEXT NOT NULL,
  topic1 TEXT,
  topic2 TEXT,
  topic3 TEXT,
  transaction_hash TEXT NOT NULL,
  transaction_log_index BIGINT, CONSTRAINT log_hash_with_index PRIMARY KEY(transaction_hash, log_index)
);

INSERT INTO logs (address, block_number, block_hash, chain, data, log_index, removed, timestamp, topic0, topic1, topic2, topic3, transaction_hash, transaction_log_index) VALUES ('0x0000000000000000000000000000000000000000', 0, '0x0000000000000000000000000000000000000000000000000000000000000000', 1, '0x', 0, FALSE, 0, '0x0000000000000000000000000000000000000000000000000000000000000000', NULL, NULL, NULL, '0x0000000000000000000000000000000000000000000000000000000000000000', NULL);