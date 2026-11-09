CREATE TABLE IF NOT EXISTS todos
(
    namespace  VARCHAR(36) NOT NULL,
    id         SERIAL      NOT NULL,
    content    TEXT        NOT NULL,
    status     VARCHAR(32) NOT NULl,
    created_at TIMESTAMP   NOT NULL,
    updated_at TIMESTAMP   NOT NULL,
    PRIMARY KEY (namespace, id)
);
