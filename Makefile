.PHONY:
.SILENT:

DECRYPT_CONFIGS:
	git-crypt unlock

RUN_APP:
	export RUST_BACKTRACE=full && \
	export RUST_LOG=purchase_validate_test=trace,warn && \
	cargo clippy && \
	cargo build --release && \
	target/release/purchase_validate_test \
		--config "./configs/test_mhouse.yml" \
		-vv