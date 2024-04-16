run: cargo run
test: cargo test
prepare: cargo sqlx prepare
init-db: ./scripts/init_db.sh
init-redis: ./scripts/init_redis.sh