.PHONY:
.SILENT:

DECRYPT_CONFIGS:
	git-crypt unlock

RUN_APP:
	export RUST_BACKTRACE=full && \
	export RUST_LOG=purchase_validate_test=trace,warn && \
	cargo clippy && \
	cargo build --release && \
	target/release/config_test_app \
		--config "./configs/test_mhouse.yml" \
		-vv