run: styling
	RUST_LOG=INFO cargo run -- run -c ${YML_FILE}

styling:
	npm run tw:gen

migrate:
	sqlx migrate add $(name) --source src/database/migrations