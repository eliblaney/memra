CREATE TABLE followers (
	id SERIAL PRIMARY KEY,
	follower_id INT NOT NULL,
	following_id INT NOT NULL,
	CONSTRAINT fk_follower
	FOREIGN KEY(follower_id)
	REFERENCES users(id),
	CONSTRAINT fk_following
	FOREIGN KEY(following_id)
	REFERENCES users(id)
)