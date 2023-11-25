run:
	RUST_LOG=INFO cargo run -- run -c ${YML_FILE}

migrate:
	sqlx migrate add $(name) --source src/database/migrations