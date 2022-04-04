CREATE TABLE IF NOT EXISTS todos
(
    id         SERIAL PRIMARY KEY,
    content    TEXT        NOT NULL,
    status     VARCHAR(32) NOT NULl,
    created_at TIMESTAMP   NOT NULL,
    updated_at TIMESTAMP   NOT NULL
);
