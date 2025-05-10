.PHONY: build shell

IMAGE=rust-dev

build:
	docker build -t ${IMAGE} .

shell:
	docker run --rm -it \
		-v ${PWD}:/home/devuser/coup \
		-w /home/devuser/coup \
		${IMAGE} bash
