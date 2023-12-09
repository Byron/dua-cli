docker_image = docker_developer_environment

help:  ## Display this help
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

always:

##@ Docker Support

interactive-developer-environment-in-docker: ## Use docker for all dependencies - run make from there
	docker build -t $(docker_image) - < etc/developer.Dockerfile
	docker run -v $$PWD:/volume -w /volume -it $(docker_image)

##@ Development

target/debug/dua: always
	cargo build

target/release/dua: always
	cargo build --release

clippy: ## Run lints with clippy
	cargo clippy -- -D warnings

check-fmt: ## Validate that the codebase is formatted correctly (but do not format it)
	cargo fmt --check

profile: target/release/dua ## run callgrind and annotate its output - linux only
	valgrind --callgrind-out-file=callgrind.profile --tool=callgrind  $< >/dev/null
	callgrind_annotate --auto=yes callgrind.profile

benchmark: target/release/dua ## see how fast things are, powered by hyperfine
	hyperfine '$<'

tests: check unit-tests journey-tests ## run all tests

check:## run cargo-check with various features
	cargo check --all
	cargo check --all-features
	cargo check --no-default-features
	cargo check --no-default-features --features tui-unix
	cargo check --no-default-features --features tui-crossplatform
	cargo check --no-default-features --features trash-move

unit-tests: ## run all unit tests
	cargo test --all
	cargo test --all --no-default-features --features trash-move

continuous-unit-tests: ## run all unit tests whenever something changes
	watchexec -w src $(MAKE) unit-tests

journey-tests: target/debug/dua ## run stateless journey tests
	./tests/stateless-journey.sh $<

continuous-journey-tests: ## run stateless journey tests whenever something changes
	watchexec $(MAKE) journey-tests

check-pre-push: check-fmt clippy tests ## Run this before pushing for a good chance that CI will be green

