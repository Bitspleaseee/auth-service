CREATE TABLE roles (

  id            INT NOT NULL,
  name          VARCHAR(20),

  PRIMARY KEY (id),
  FOREIGN KEY(id) REFERENCES users(id)
);