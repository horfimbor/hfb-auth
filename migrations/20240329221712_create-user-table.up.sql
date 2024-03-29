create table account
(
    uuid   uuid default uuid() not null,
    pseudo varchar(32)         not null,
    role   text                null,
    constraint account_pk
        primary key (uuid),
    constraint account_pk_2
        unique (pseudo)
)
    collate = utf8mb4_unicode_ci;

