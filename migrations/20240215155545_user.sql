-- Add migration script here
CREATE TABLE IF NOT EXISTS users 
(
	id uuid NOT NULL UNIQUE,
	name VARCHAR(64) NOT NULL UNIQUE,
	email VARCHAR(256) NOT NULL UNIQUE,
    password VARCHAR(256) NOT NULL,
	PRIMARY KEY (id)
);