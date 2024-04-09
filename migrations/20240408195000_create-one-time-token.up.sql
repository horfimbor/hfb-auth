create table token_one_time
(
    application_id uuid                     not null,
    account_id     uuid                     not null,
    token          uuid default uuid() not null,
    constraint `token-one-time_account_id_fk`
        foreign key (account_id) references account (uuid),
    constraint `token-one-time_application_id_fk`
        foreign key (application_id) references application (uuid)
)
    collate = utf8mb4_unicode_ci;
