CREATE TABLE division_updates (
  id SERIAL PRIMARY KEY,
  division_id INTEGER REFERENCES divisions(id) NOT NULL,
  publication_updated TEXT NOT NULL
)
