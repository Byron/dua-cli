docker_image = docker_developer_environment

help:
	$(info -Targets -----------------------------------------------------------------------------)
	$(info -Development Targets -----------------------------------------------------------------)
	$(info lint                         | run lints with clippy)
	$(info benchmark                    | just for fun, really)
	$(info profile                      | only on linux - run callgrind and annotate it)
	$(info tests                        | run all tests)
	$(info unit-tests                   | run all unit tests)
	$(info continuous-unit-tests        | run all unit tests whenever something changes)
	$(info journey-tests                | run all stateless journey test)
	$(info continuous-journey-tests     | run all stateless journey test whenever something changes)
	$(info -- Use docker for all dependencies - run make interactively from there ----------------)
	$(info interactive-developer-environment-in-docker | gives you everything you need to run all targets)

always:

interactive-developer-environment-in-docker:
	docker build -t $(docker_image) - < etc/developer.Dockerfile
	docker run -v $$PWD:/volume -w /volume -it $(docker_image)

target/debug/dua: always
	cargo build

target/release/dua: always
	cargo build --release

lint:
	cargo clippy

profile: target/release/dua
	valgrind --callgrind-out-file=callgrind.profile --tool=callgrind  $< >/dev/null
	callgrind_annotate --auto=yes callgrind.profile

benchmark: target/release/dua
	hyperfine '$<'

tests: unit-tests journey-tests

unit-tests:
	cargo test --bin dua

continuous-unit-tests:
	watchexec $(MAKE) unit-tests

journey-tests: target/debug/dua
	./tests/stateless-journey.sh $<

continuous-journey-tests:
	watchexec $(MAKE) journey-tests

