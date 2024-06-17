CREATE DATABASE devel 
  CHARACTER SET = 'utf8'
  COLLATE = 'utf8_general_ci';

create table clients(
    id BIGINT unsigned NOT NULL AUTO_INCREMENT,
    first_name VARCHAR(50) NOT NULL,
    last_name VARCHAR(50) NOT NULL,
    email VARCHAR(255) NOT NULL,
    PRIMARY KEY (id)
);

create table credentials(
    id BIGINT unsigned NOT NULL AUTO_INCREMENT,
    client_id BIGINT unsigned NOT NULL,

    client_key MEDIUMTEXT NOT NULL,
    password varchar(50) NOT NULL,
    PRIMARY KEY (id)
);

create OR REPLACE USER admin@localhost 
IDENTIFIED BY 'admin123';

GRANT ALL PRIVILEGES ON clients
TO 'admin'@'localhost';

GRANT ALL PRIVILEGES ON passwords
TO 'admin'@'localhost';


-- BASIC QUERIES --

INSERT INTO clients (first_name, last_name, email)
VALUES ('Tomek', 'Świątkowski', 'tomek@schronnet');

INSERT INTO credentials (client_id, client_key, password)
SELECT id, "SecretKey", "Piernik" FROM clients
where first_name like 'Tomek' and last_name like 'Świątkowski';