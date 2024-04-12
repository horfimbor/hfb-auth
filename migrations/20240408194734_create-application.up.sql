
create table application
(
    uuid uuid default uuid() not null,
    name varchar(255)               not null,
    host varchar(255)               not null,
    app_key varchar(255)            not null,
    constraint application_pk
        primary key (uuid),
    constraint application_pk_2
        unique (name),
    constraint application_pk_3
        unique (host)
)
    collate = utf8mb4_unicode_ci;

INSERT INTO application (uuid, name, host, app_key) VALUES (DEFAULT, 'localhost', 'http://galaxy.localhost:8001', 'azerty123456');
INSERT INTO application (uuid, name, host, app_key) VALUES (DEFAULT, 'minikube', 'http://galaxy.hfm', 'someSecure_token');