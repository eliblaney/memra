CREATE TABLE notifications (
	id SERIAL PRIMARY KEY,
	user_id INT NOT NULL,
	message TEXT NOT NULL,
	icon BYTEA,
	CONSTRAINT fk_notificationuser
	FOREIGN KEY(user_id)
	REFERENCES users(id)
)