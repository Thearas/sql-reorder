-- transaction
begin;
update X set a=100 where id=1;
update X set a=100 where id=2;
update X set a=100 where id=8;
commit;