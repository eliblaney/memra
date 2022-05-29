CREATE TABLE history (
	id SERIAL PRIMARY KEY,
	user_id INT NOT NULL,
	card_id INT NOT NULL,
	ts TIMESTAMP,
	num_confident INT NOT NULL DEFAULT 0,
	num_correct INT NOT NULL DEFAULT 0,
	num_wrong INT NOT NULL DEFAULT 0,
	CONSTRAINT fk_historyuser
	FOREIGN KEY(user_id)
	REFERENCES users(id),
	CONSTRAINT fk_historycard
	FOREIGN KEY(card_id)
	REFERENCES cards(id)
)