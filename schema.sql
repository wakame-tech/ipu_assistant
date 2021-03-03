CREATE TABLE users (
  id text NOT NULL PRIMARY KEY,
  name text,
  count integer
);

CREATE TABLE events (
  id SERIAL,
  event text PRIMARY KEY NOT NULL,
  cron text NOT NULL
);