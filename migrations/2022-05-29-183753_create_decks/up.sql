CREATE TABLE decks (
	id SERIAL PRIMARY KEY,
	user_id INT NOT NULL,
	visibility BOOLEAN DEFAULT 'f',
	name VARCHAR NOT NULL,
	image BYTEA,
	CONSTRAINT fk_deckuser
	FOREIGN KEY(user_id)
	REFERENCES users(id)
)