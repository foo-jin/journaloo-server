-- Your SQL goes here
CREATE TABLE entries (
  id          SERIAL PRIMARY KEY,
  journey_id   SERIAL REFERENCES journeys (id),
  created     TIMESTAMP        DEFAULT now(),
  archived    BOOLEAN NOT NULL DEFAULT 'f',
  description VARCHAR,
  coordinates VARCHAR,
  location    VARCHAR
)