CREATE TABLE settings (
	id SERIAL PRIMARY KEY,
	user_id INT NOT NULL,
	public_profile BOOLEAN DEFAULT 'f',
	avatar BYTEA,
	CONSTRAINT fk_settingsuser
	FOREIGN KEY(user_id)
	REFERENCES users(id)
)