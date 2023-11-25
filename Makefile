run:
	RUST_LOG=INFO cargo run -- run -c ${YML_FILE}

styling:
	npm run watch

dev: styling run

migrate:
	sqlx migrate add $(name) --source src/database/migrations