CREATE TABLE course_subscriptions (
	id SERIAL PRIMARY KEY,
	user_id INT NOT NULL,
	course_id INT NOT NULL,
	CONSTRAINT fk_coursesubuser
	FOREIGN KEY(user_id)
	REFERENCES users(id),
	CONSTRAINT fk_coursesubcourse
	FOREIGN KEY(course_id)
	REFERENCES courses(id)
)