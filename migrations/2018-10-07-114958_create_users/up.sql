
CREATE TABLE users (

  id            INT AUTO_INCREMENT,
  email         VARCHAR(255) NOT NULL UNIQUE,
  username      VARCHAR(20) NOT NULL UNIQUE,
  password      TINYTEXT NOT NULL,
  banned        BOOLEAN DEFAULT FALSE,
  verified      BOOLEAN DEFAULT FALSE,
  email_token   VARCHAR(255),

  PRIMARY KEY (id)
);

CREATE TABLE roles (

  id            INT NOT NULL,
  name          VARCHAR(20),

  PRIMARY KEY (id),
  FOREIGN KEY(id) REFERENCES users(id)
);