-- Your SQL goes here
CREATE TABLE journeys (
  id        SERIAL PRIMARY KEY,
  user_id    SERIAL REFERENCES users (id),
  title     VARCHAR   NOT NULL,
  archived  BOOLEAN   NOT NULL DEFAULT 'f',
  start_date TIMESTAMP NOT NULL DEFAULT now(),
  end_date   TIMESTAMP          DEFAULT NULL
)