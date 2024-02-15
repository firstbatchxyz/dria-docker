.PHONY: build-hollowdb-api run-hollowdb-api build-hollowdb-wait run-hollowdb-wait build-hnsw run-hnsw

## scripts for local development

######### HOLLOWDB ##########
build-hollowdb: 
	docker build ./hollowdb -t hollowdb-tests

run-hollowdb:
	docker run \
	-e CONTRACT_TXID=WbcY2a-KfDpk7fsgumUtLC2bu4NQcVzNlXWi13fPMlU \
	-v ${PWD}/wallet.json:/app/config/wallet.json:ro \
	hollowdb-tests

build-hollowdb: 
	docker build ./hollowdb -t hollowdb-tests --push

####### HOLLOWDB WAIT #######
build-hollowdb-wait:
	docker build ./hollowdb_wait -t hollowdb-wait-for

run-hollowdb-wait:
	docker run hollowdb-wait-for

run-hollowdb-wait:
	docker build ./hollowdb_wait -t hollowdb-wait-for --push

######### DRIA HNSW #########
build-hnsw:
	docker build ./dria_hnsw -t hollowdb-wait-for

run-hnsw:
	docker run hollowdb-wait-for
