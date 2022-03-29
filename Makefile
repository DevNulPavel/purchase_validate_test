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
		--configs "./configs/test_mhouse.yml"
		# -vv

SERVER_LOAD_TEST:
	export RUST_BACKTRACE=full && \
	export RUST_LOG=purchase_validate_test=trace,warn && \
	cargo clippy && \
	cargo build --release && \
	target/release/server_loadtest_app \
		--configs "./configs/test_mhouse.yml" \
		--requests-parallel-threads 10 \
		--requests-per-thread 20