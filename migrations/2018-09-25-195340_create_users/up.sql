CREATE TABLE users (

  id            INT AUTO_INCREMENT,
  email         VARCHAR(255) NOT NULL UNIQUE,
  username      VARCHAR(20) NOT NULL,
  password      TINYTEXT NOT NULL,
  banned        BOOLEAN DEFAULT FALSE,
  verified      BOOLEAN DEFAULT FALSE,
  email_token   VARCHAR(255),

  PRIMARY KEY (id)
);