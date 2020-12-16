
container:
	docker build -t guedes233/gesticle-builder .

container-build:
	docker run --rm -v $(PWD)/..:/home/circleci/code -ti guedes233/gesticle-builder:latest\
		sh -c "cd /home/circleci/project && cp -r /home/circleci/code/* . ; cargo deb -p gesticled"
