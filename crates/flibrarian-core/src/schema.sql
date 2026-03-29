SET autoinstall_known_extensions=1;
SET autoload_known_extensions=1;
PRAGMA enable_object_cache;

CREATE TYPE IF NOT EXISTS archive_status AS ENUM ('indexing', 'indexed');

CREATE SEQUENCE IF NOT EXISTS archives_id_seq;

CREATE TABLE IF NOT EXISTS archives (
    id UINTEGER DEFAULT nextval('archives_id_seq') PRIMARY KEY,
    name VARCHAR UNIQUE,
    status archive_status DEFAULT 'indexing'
);

CREATE TABLE IF NOT EXISTS books (
    id UINTEGER PRIMARY KEY,
    title VARCHAR,
    genres JSON,
    date VARCHAR,
    lang VARCHAR DEFAULT '',
    file_size UBIGINT DEFAULT 0,
    sequence VARCHAR,
    archive_id UINTEGER,
    FOREIGN KEY (archive_id) REFERENCES archives(id)
);

CREATE TABLE IF NOT EXISTS authors (
    id VARCHAR PRIMARY KEY,
    first_name VARCHAR,
    middle_name VARCHAR,
    last_name VARCHAR,
    nickname VARCHAR
);

CREATE TABLE IF NOT EXISTS books_authors (
    book_id UINTEGER,
    author_id VARCHAR,
    PRIMARY KEY (book_id, author_id),
    FOREIGN KEY (book_id) REFERENCES books(id),
    FOREIGN KEY (author_id) REFERENCES authors(id)
);
