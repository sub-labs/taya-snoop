CREATE TABLE logs (
  address TEXT NOT NULL,
  block_number BIGINT NOT NULL,
  block_hash TEXT NOT NULL,
  chain BIGINT NOT NULL,
  data TEXT NOT NULL,
  from_address TEXT NOT NULL,
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