CREATE TABLE coursedecks (
	id SERIAL PRIMARY KEY,
	course_id INT NOT NULL,
	deck_id INT NOT NULL,
	CONSTRAINT fk_coursedeckcourse
	FOREIGN KEY(course_id)
	REFERENCES courses(id),
	CONSTRAINT fk_coursedeckdeck
	FOREIGN KEY(deck_id)
	REFERENCES decks(id)
)