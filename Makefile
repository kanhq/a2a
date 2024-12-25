
local_a2a: 
	cargo build --release --bin a2a
	cp /tmp/a2a/release/a2a ${HOME}/.local/bin/a2a

docker_base:
	docker build -t registry.cn-beijing.aliyuncs.com/kanhq-dev/a2a:base -f Dockerfile.base .
	docker push registry.cn-beijing.aliyuncs.com/kanhq-dev/a2a:base

docker:
	docker build -t registry.cn-beijing.aliyuncs.com/kanhq-dev/a2a:serve_latest .
	docker push registry.cn-beijing.aliyuncs.com/kanhq-dev/a2a:serve_latest