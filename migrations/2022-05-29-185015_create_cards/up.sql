CREATE TABLE cards (
	id SERIAL PRIMARY KEY,
	user_id INT NOT NULL,
	deck_id INT NOT NULL,
	front BYTEA,
	back BYTEA,
	CONSTRAINT fk_carduser
	FOREIGN KEY(user_id)
	REFERENCES users(id),
	CONSTRAINT fk_carddeck
	FOREIGN KEY(deck_id)
	REFERENCES decks(id)
)