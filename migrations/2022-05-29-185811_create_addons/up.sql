CREATE TABLE addons (
	id SERIAL PRIMARY KEY,
	user_id INT NOT NULL,
	visibility BOOLEAN DEFAULT 'f',
	name VARCHAR NOT NULL,
	description TEXT NOT NULL,
	data BYTEA,
	CONSTRAINT fk_addonuser
	FOREIGN KEY(user_id)
	REFERENCES users(id),
	CONSTRAINT len_addondescription
	CHECK (length(description) < 1000)
)