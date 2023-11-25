CREATE TABLE IF NOT EXISTS account_books (
    account_id BYTEA REFERENCES accounts(id),
    book_id BYTEA REFERENCES books(id)
);