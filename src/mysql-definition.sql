create database if not exists stocks_rs;
use stocks_rs;
drop database stocks_rs;
create user if not exists 'rust'@'localhost' identified by 'password';
grant all on stocks_rs.* to 'rust'@'localhost';

create table if not exists tsla_raw (
	entry_date DATE not null primary key,
    close_value decimal(11, 2) not null,
    split_coefficient decimal(4, 2) not null
);

create table if not exists test1 (
	entry_date DATE not null primary key,
    close_value decimal(11, 2) not null,
    split_coefficient decimal(4, 2) not null
);

insert into test1 (entry_date, close_value, split_coefficient) values ("2020-05-03", 200, 1.0);
insert into test1 (entry_date, close_value, split_coefficient) values ("2020-05-01", 100, 2.0);
insert into test1 (entry_date, close_value, split_coefficient) values ("2020-04-01", 400, 1.0);

select * from tsla_raw;
select * from tsla_adjusted;
select * from goog_raw;
select * from goog_adjusted;
select * from goog_200avg;
drop table test1;
Insert into goog_200avg (entry_date, close_value) values('2020-10-10', (with temp as ( select close_value from tsla_adjusted where entry_date <= '2020-10-10' order by entry_date desc limit 200) select avg(close_value) from temp));
select * from tsla_raw order by entry_date DESC;
delete from tsla_raw where entry_date = "2021-04-09";
drop table goog_200avg;