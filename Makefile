
######### DRIA HNSW #########
.PHONY: build-hnsw run-hnsw push-hnsw

build-hnsw:
	docker build ./dria_hnsw -t dria-hnsw

run-hnsw:
	docker run \
	-e CONTRACT_ID=WbcY2a-KfDpk7fsgumUtLC2bu4NQcVzNlXWi13fPMlU \
	-e REDIS_URL=redis://default:redispw@localhost:6379 \
	-p 8080:8080 \
	dria-hnsw

push-hnsw:
	docker buildx build \
	--platform=linux/amd64,linux/arm64,linux/arm ./dria_hnsw \
	-t firstbatch/dria-hnsw:latest \
	--builder=dria-builder --push

######### HOLLOWDB ##########
.PHONY: build-hollowdb run-hollowdb push-hollowdb

build-hollowdb: 
	docker build ./hollowdb -t dria-hollowdb

run-hollowdb:
	docker run \
	-e CONTRACT_ID=WbcY2a-KfDpk7fsgumUtLC2bu4NQcVzNlXWi13fPMlU \
	-e REDIS_URL=redis://default:redispw@localhost:6379 \
	-p 3030:3030 \
	dria-hollowdb

push-hollowdb: 
	docker build ./hollowdb -t firstbatch/dria-hollowdb:latest --push

####### HOLLOWDB WAIT #######
.PHONY: build-hollowdb-wait run-hollowdb-wait push-hollowdb-wait

build-hollowdb-wait:
	docker build ./hollowdb_wait -t hollowdb-wait-for

run-hollowdb-wait:
	docker run hollowdb-wait-for

push-hollowdb-wait:
	docker build ./hollowdb_wait -t firstbatch/dria-hollowdb-wait-for:latest --push

########### REDIS ###########
pull-redis:
	docker pull redis:alpine

run-redis:
	docker run redis:alpine --name dria-redis -p 6379:6379

########## BUILDER ##########
.PHONY: dria-builder

# see: https://docs.docker.com/build/building/multi-platform/#cross-compilation
dria-builder:
	docker buildx create --name dria-builder --bootstrap --use
