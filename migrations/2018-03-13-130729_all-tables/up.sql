-- Your SQL goes here
CREATE TABLE users (
  id       SERIAL PRIMARY KEY,
  username VARCHAR   NOT NULL,
  email    VARCHAR   NOT NULL,
  password VARCHAR   NOT NULL,
  date     TIMESTAMP NOT NULL DEFAULT now()
);

CREATE TABLE journeys (
  id         SERIAL PRIMARY KEY,
  user_id    SERIAL REFERENCES users (id),
  title      VARCHAR   NOT NULL,
  archived   BOOLEAN   NOT NULL DEFAULT 'f',
  start_date TIMESTAMP NOT NULL DEFAULT now(),
  end_date   TIMESTAMP          DEFAULT NULL
);

CREATE TABLE entries (
  id          SERIAL PRIMARY KEY,
  journey_id  SERIAL REFERENCES journeys (id),
  user_id     SERIAL REFERENCES users (id),
  created     TIMESTAMP NOT NULL DEFAULT now(),
  archived    BOOLEAN   NOT NULL DEFAULT 'f',
  description VARCHAR,
  coordinates VARCHAR,
  location    VARCHAR
)