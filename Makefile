COREUM_CHAIN_ID="coreum-testnet-1"
COREUM_DENOM=utestcore
COREUM_NODE=https://full-node.testnet-1.coreum.dev:26657
COREUM_VERSION="{Cored version}"
COREUM_CHAIN_ID_ARGS=--chain-id=$(COREUM_CHAIN_ID)
COREUM_NODE_ARGS=--node=$(COREUM_NODE)
COREUM_HOME=$(HOME)/.core/"$(COREUM_CHAIN_ID)"
COREUM_BINARY_NAME=$(shell arch | sed s/aarch64/cored-linux-arm64/ | sed s/x86_64/cored-linux-amd64/)

DEV_WALLET=dev-wallet
CODE_ID=1178

_CONTRACT_ADDRESS_=testcore1rt5khxlnt9yh0wqwdwvdu0y9lfheapx3jh6w6hqav6782fr3xedsyywkxl
_WALLET_ADDRESS_=testcore1xhvglxz55w0uy73t5lxhypt8leud9wsd92ccjq

.PHONY: dev test add_account build deploy check keys q fund instantiate contract_address

dev:
	@echo "${PWD}"
	@echo `basename "${PWD}"`
	cargo build

test:
	cargo test -- --nocapture

add_account:
	cored keys add ${DEV_WALLET} --recover

build:
	docker run --rm -v "${PWD}":/code \
	--mount type=volume,source=`basename "${PWD}"`_cache,target=/code/target \
	--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
	cosmwasm/optimizer:0.15.0

deploy:
	@RES=$$(cored tx wasm store artifacts/grant_dao.wasm --from ${DEV_WALLET} --gas auto --gas-adjustment 1.3 -y -b block --output json "$(COREUM_NODE_ARGS)" "$(COREUM_CHAIN_ID_ARGS)") ; \
	echo $$RES ; \
	CODE_ID=$$(echo $$RES | jq -r '.logs[0].events[-1].attributes[-1].value') ; \
	echo "Code ID: $$CODE_ID"

check:
	cored q wasm code-info $(CODE_ID) $(COREUM_NODE_ARGS) $(COREUM_CHAIN_ID_ARGS)

keys:
	cored keys list

q:
	cored- q bank balances $(_WALLET_ADDRESS_)

fund:
	cored tx bank send alice $(_WALLET_ADDRESS_) 10000000udevcore --broadcast-mode=block
	
instantiate:
	cored tx wasm instantiate $(CODE_ID) \
	"{\"members\":[{\"address\":\"$(_WALLET_ADDRESS_)\", \"weight\":\"10\"}]}" \
	--amount="10000000$(COREUM_DENOM)" --no-admin --label "Grant Dao" --from ${DEV_WALLET} --gas auto --gas-adjustment 1.3 -b block -y $(COREUM_NODE_ARGS) $(COREUM_CHAIN_ID_ARGS)

contract_address:
	@echo $(_CONTRACT_ADDRESS_)

propose:
	cored tx wasm execute $(_CONTRACT_ADDRESS_) \
	"{\"propose\": {\"title\":\"YOUR_TITLE_HERE\", \"description\":\"YOUR_DESCRIPTION_HERE\"}}" \
	--from ${DEV_WALLET} --gas auto --gas-adjustment 1.3 -b block -y $(COREUM_NODE_ARGS) $(COREUM_CHAIN_ID_ARGS)


vote:
	cored tx wasm execute $(_CONTRACT_ADDRESS_) \
	"{\"vote\": {\"proposal_id\":0, \"approve\":true}}" \
	--from ${DEV_WALLET} --gas auto --gas-adjustment 1.3 -b block -y $(COREUM_NODE_ARGS) $(COREUM_CHAIN_ID_ARGS)

execute:
	cored tx wasm execute $(_CONTRACT_ADDRESS_) \
	"{\"execute\": {\"proposal_id\":0}}" \
	--from ${DEV_WALLET} --gas auto --gas-adjustment 1.3 -b block -y $(COREUM_NODE_ARGS) $(COREUM_CHAIN_ID_ARGS)
