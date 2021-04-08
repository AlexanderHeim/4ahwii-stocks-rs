create database if not exists stocks_rs;
use stocks_rs;

create user if not exists 'rust'@'localhost' identified by 'password';
grant all on stocks_rs.* to 'rust'@'localhost';

create table if not exists tsla_raw (
	entry_date DATE not null primary key,
    close_value decimal(11, 2) not null,
    split_coefficient decimal(4, 2) not null
);
