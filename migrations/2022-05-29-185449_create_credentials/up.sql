CREATE TABLE credentials (
	id SERIAL PRIMARY KEY,
	user_id INT NOT NULL,
	email VARCHAR NOT NULL,
	password VARCHAR NOT NULL,
	CONSTRAINT fk_credentialsuser
	FOREIGN KEY(user_id)
	REFERENCES users(id)
)