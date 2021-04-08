create database if not exists stocks_rs;
use stocks_rs;

create table if not exists raw_tsla (
	entry_date DATE not null primary key,
    close_value decimal(11, 2) not null,
    split_coefficient decimal(4, 2) not null
);

drop table raw_tsla;