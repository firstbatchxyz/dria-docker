.PHONY: build-micro-api run-micro-api build-micro-wait run-micro-wait build-hnsw run-hnsw

## scripts for local development

######### MICRO API ##########
build-micro-api: 
	docker build ./micro_api -t micro-tests

run-micro-api:
	docker run \
	-e CONTRACT_TXID=WbcY2a-KfDpk7fsgumUtLC2bu4NQcVzNlXWi13fPMlU \
	-v ${PWD}/wallet.json:/app/config/wallet.json:ro \
	micro-tests

# TODO: push

##### MICRO API WAIT-FOR #####
build-micro-wait:
	docker build ./micro_api_wait -t micro-wait-for

run-micro-wait:
	docker build ./micro_api_wait -t micro-wait-for

######### DRIA HNSW #########
build-hnsw:
	docker build ./dria_hnsw -t micro-wait-for

run-hnsw:
	docker build ./micro_api_wait -t micro-wait-for
