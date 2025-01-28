CREATE TABLE tokens (
  address TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  symbol TEXT NOT NULL,
  decimals BIGINT NOT NULL
);