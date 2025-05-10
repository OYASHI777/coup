.PHONY: build shell

IMAGE=rust-dev

build:
	docker build -t ${IMAGE} .

shell:
	docker run --rm -it \
		-v ${PWD}:/home/devuser/coup \
		-v "${HOME}/.gitconfig":/home/devuser/.gitconfig:ro \
		-v "${HOME}/.git-credentials":/home/devuser/.git-credentials:ro \
		-w /home/devuser/coup \
		${IMAGE} \
		bash
