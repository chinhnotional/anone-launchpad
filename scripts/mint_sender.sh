#!/bin/bash

NODE="http://65.108.128.139:2281"
CHAINID="anone-testnet-1"
SLEEP_TIME="15s"
CONTRACT_MINTER="one16rhma65e6k4yclpqp3652w0cgh6l6engdrhrusy5ftyjsvrhgkwqkncrtu"
CONTRACT_AN721="one14h9jdcg8ke8vnech25erz8v98utc6vaneaph3rlpx627nas4jt3s7jv0ur"


# CHANGE ONLY THIS
OWNER="Developer"
MODEL_ID="$1"
SIZE="$2"
MINT="{\"mint\": {\"model_id\":\"$MODEL_ID\", \"size\":\"$SIZE\"}}"

echo $MINT

RES=$(anoned tx wasm execute "$CONTRACT_MINTER" "$MINT" --from "$OWNER" -y --output json --chain-id "$CHAINID" --node "$NODE" --gas 35000000 --fees 0uan1 -y --output json)
echo $RES

TXHASH=$(echo $RES | jq -r .txhash)

echo $TXHASH

# sleep for chain to update
sleep "$SLEEP_TIME"

RAW_LOG=$(anoned query tx "$TXHASH" --chain-id "$CHAINID" --node "$NODE" -o json | jq -r .raw_log)

echo $RAW_LOG

