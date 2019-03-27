all:
	cargo rustc -p preload -- -C link-arg=-Wl,--version-script=ld.version
	cargo build --all --exclude preload

clean:
	cargo clean
