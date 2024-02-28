-- Your SQL goes here
CREATE TABLE employees (
  id SERIAL PRIMARY KEY,
  name VARCHAR NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT now()
)