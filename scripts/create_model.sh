#!/bin/bash

NODE="http://65.108.128.139:2281"
CHAINID="anone-testnet-1"
SLEEP_TIME="15s"
CONTRACT_MINTER="one1s5xvg8hkx7k35xzf2cp9d6drkmw5k9qaxc4m7jhzjgt0p2648n3qfaxamd"
CONTRACT_AN721="one1dy3j893n3jnnq5se74cmgpsqu0dxm35736tqk9yf6g8wkjnknuqquxkaw6"

# CHANGE ONLY THIS
OWNER="Developer"
MODEL_ID="$1"
MODEL_URI="ipfs/bafybeiaivv62j7jxlkahxobfr5io7h2j56obw5mojljho2ybg7zhah2eue/galaxyfcnCU3/2"

CREATE_MODEL="{\"create_model\": {\"model_id\": \"$MODEL_ID\", \"model_uri\":\"$MODEL_URI\"}}"

echo $CREATE_MODEL

RES=$(anoned tx wasm execute "$CONTRACT_MINTER" "$CREATE_MODEL" --from "$OWNER" -y --output json --chain-id "$CHAINID" --node "$NODE" --gas 35000000 --fees 0uan1 -y --output json)
echo $RES

TXHASH=$(echo $RES | jq -r .txhash)

echo $TXHASH

# sleep for chain to update
sleep "$SLEEP_TIME"

RAW_LOG=$(anoned query tx "$TXHASH" --chain-id "$CHAINID" --node "$NODE" -o json | jq -r .raw_log)

echo $RAW_LOG