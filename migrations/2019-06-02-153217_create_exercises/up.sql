CREATE TABLE exercises (
    id VARCHAR PRIMARY KEY,
    title VARCHAR NOT NULL,
    body TEXT NOT NULL,
    topic VARCHAR,
    created_on BIGINT NOT NULL,
    modified_on BIGINT NOT NULL
)
