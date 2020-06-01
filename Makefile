all:
	@cargo build
	@for f in target/debug/bench-*; do ln -fs $$f; done
	@rm bench-*.d
