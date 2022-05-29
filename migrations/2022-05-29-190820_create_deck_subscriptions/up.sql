CREATE TABLE deck_subscriptions (
	id SERIAL PRIMARY KEY,
	user_id INT NOT NULL,
	deck_id INT NOT NULL,
	CONSTRAINT fk_decksubuser
	FOREIGN KEY(user_id)
	REFERENCES users(id),
	CONSTRAINT fk_decksubdeck
	FOREIGN KEY(deck_id)
	REFERENCES decks(id)
)