-- Your SQL goes here
CREATE TABLE journeys (
  id        SERIAL PRIMARY KEY,
  userid    SERIAL REFERENCES users (id),
  title     VARCHAR   NOT NULL,
  archived  BOOLEAN   NOT NULL DEFAULT 'f',
  startdate TIMESTAMP NOT NULL DEFAULT NOW(),
  enddate   TIMESTAMP          DEFAULT NULL
)