create database if not exists stocks_rs;
use stocks_rs;

create user if not exists 'rust'@'localhost' identified by 'password';
grant all on stocks_rs.* to 'rust'@'localhost';

create table if not exists tsla_raw (
	entry_date DATE not null primary key,
    close_value decimal(11, 2) not null,
    split_coefficient decimal(4, 2) not null
);

select * from tsla_raw order by entry_date DESC;
delete from tsla_raw where entry_date = "2021-04-08";
drop table tsla_raw;