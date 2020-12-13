
container:
	docker build -t guedes233/gesticle-builder .

container-build:
	docker run --rm -v $PWD:/home/circleci/code -ti guedes233/gesticle-builder:latest\
		cp -r ../code . && cargo build
