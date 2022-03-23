.PHONY:
.SILENT:

DECRYPT_CONFIGS:
	git-crypt unlock

RUN_APP:
	export RUST_LOG=file_upload_proxy=trace,warn && \
	cargo clippy && \
	cargo build --release && \
	target/release/file_upload_proxy \
		--config "./configs/test.yaml"