CREATE TABLE users (
	id SERIAL PRIMARY KEY,
	username VARCHAR NOT NULL,
	real_name VARCHAR,
	verified BOOLEAN NOT NULL DEFAULT 'f'
)