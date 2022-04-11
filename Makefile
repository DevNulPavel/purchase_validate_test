.PHONY:
.SILENT:

DECRYPT_CONFIGS:
	git-crypt unlock

VALIDATE_PURCHASES:
	export RUST_BACKTRACE=full && \
	export RUST_LOG=purchase_validate_test=trace,warn && \
	cargo clippy && \
	cargo build --release && \
	target/release/config_test_app \
		--configs "./configs/prod_mhouse.yml"
		# -vv
		# "./configs/test_mhouse_win_only.yml" "./configs/test_island2_win_only.yml"
		# "./configs/test_mhouse.yml" "./configs/test_island2.yml"
		# "./configs/prod_island2.yml" "./configs/prod_mhouse.yml"

SERVER_LOAD_TEST:
	export RUST_BACKTRACE=full && \
	export RUST_LOG=purchase_validate_test=trace,warn && \
	cargo clippy && \
	cargo build --release && \
	target/release/server_loadtest_app \
		--configs "./configs/test_mhouse.yml" \
		--requests-parallel-threads 20 \
		--requests-per-thread 50