TARGET_DIR=$(shell cargo metadata --no-deps --format-version 1  | jq -r ".target_directory")
CURRENT_VERSION=$(shell cargo metadata --no-deps --format-version 1  | jq -r ".packages[0].version")
EXE_EXT=
SO_EXT=.so
ifeq ($(shell uname -s),"Windows_NT")
EXE_EXT=.exe
SO_EXT=.dll
endif

BINARY_NAME=a2a${EXE_EXT}
NODEJS_BINARY_NAME=liba2a_nodejs${SO_EXT}
PYTHON_BINARY_NAME=lib_a2apy${SO_EXT}

local_a2a: 
	cargo build --release --bin a2a
	cp /tmp/a2a/release/a2a ${HOME}/.local/bin/a2a

docker_base:
	docker build -t registry.cn-beijing.aliyuncs.com/kanhq-dev/a2a:base -f Dockerfile.base .
	docker push registry.cn-beijing.aliyuncs.com/kanhq-dev/a2a:base

docker:
	docker build -t registry.cn-beijing.aliyuncs.com/kanhq-dev/a2a:serve_latest .
	docker push registry.cn-beijing.aliyuncs.com/kanhq-dev/a2a:serve_latest

dist: FORCE
	cargo build --release
	rm -rf dist
	mkdir dist 

	mkdir dist/bin
	cp ${TARGET_DIR}/release/${BINARY_NAME} dist/bin/

	mkdir dist/python
	cp ${TARGET_DIR}/release/${PYTHON_BINARY_NAME} dist/python/

	mkdir dist/nodejs
	cp ${TARGET_DIR}/release/${NODEJS_BINARY_NAME} dist/nodejs/
	cp bindings/nodejs/package.json dist/nodejs/
	cp bindings/nodejs/*.ts dist/nodejs/
	cp bindings/nodejs/*.js dist/nodejs/
	sed -i "s/\"version\": \"[0-9]\+\.[0-9]\+\.[0-9]\+\"/\"version\": \"${CURRENT_VERSION}\"/" dist/nodejs/package.json

.PHONY: FORCE
FORCE: