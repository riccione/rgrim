.PHONY: test check-hook check

# Run Cargo test suite
test:
	cargo test

# Run the git pre-commit hook script directly
check-hook:
	@if [ ! -f .git/hooks/pre-commit ]; then \
		echo "Error: .git/hooks/pre-commit does not exist."; \
		exit 1; \
	fi
	@if [ ! -x .git/hooks/pre-commit ]; then \
		echo "Warning: .git/hooks/pre-commit is not executable. Running via shell..."; \
		sh .git/hooks/pre-commit; \
	else \
		./.git/hooks/pre-commit; \
	fi

# Run tests and pre-commit hook together
check: test check-hook
