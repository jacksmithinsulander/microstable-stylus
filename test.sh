#!/bin/bash

# Load variables from .env file
set -o allexport
source .env
set +o allexport

# Build ShUSD
echo "🛠️ Building ShUSD..."
cargo build --release --target wasm32-unknown-unknown --features sh-usd

# Deploy ShUSD
echo "🚀 Deploying ShUSD..."
SH_USD_ADDRESS=$(cargo stylus deploy \
    --private-key=$PRIVATE_KEY \
    --wasm-file target/wasm32-unknown-unknown/release/libmicrostable.wasm \
    --no-verify 2>/dev/null | grep "deployed code at address:" | awk '{print $5}' | tr -d '[:space:]' | tr -d '\r' | sed 's/\x1b\[[0-9;]*m//g')

# Check address length
if [ ${#SH_USD_ADDRESS} -ne 42 ]; then
    echo "Error: SH_USD_ADDRESS has incorrect length: ${#SH_USD_ADDRESS}"
    exit 1
fi

# Check the ShUSD deployment
echo "🤓 Controlling ShUSD"
if [ -z "$SH_USD_ADDRESS" ]; then
    echo "❌ Failed to extract ShUSD contract address."
    exit 1
else
    SH_NAME_TEST=$(cast call $SH_USD_ADDRESS "name()(string)" --rpc-url $RPC_URL)
    if [ $SH_NAME_TEST != "Shafu USD" ]; then
        echo "❌ Seems that token name is wrong. Received: $SH_NAME_TEST"
        exit 1
    else
        echo "✅ ShUSD Got deployed as intended, at $SH_USD_ADDRESS"
    fi
fi

# Build WETH
echo "🛠️ Building WETH..."
cargo build --release --target wasm32-unknown-unknown --features test-weth

# Deploy WETH
echo "🚀 Deploying WETH..."
WETH_ADDRESS=$(cargo stylus deploy \
    --private-key=$PRIVATE_KEY \
    --wasm-file target/wasm32-unknown-unknown/release/libmicrostable.wasm \
    --no-verify 2>/dev/null | grep "deployed code at address:" | awk '{print $5}' | tr -d '[:space:]' | tr -d '\r' | sed 's/\x1b\[[0-9;]*m//g')

# Check address length
if [ ${#WETH_ADDRESS} -ne 42 ]; then
    echo "Error: WETH_ADDRESS has incorrect length: ${#WETH_ADDRESS}"
    exit 1
fi

# Check the WETH deployment
echo "🤓 Controlling WETH"
if [ -z "$WETH_ADDRESS" ]; then
    echo "❌ Failed to extract WETH contract address."
    exit 1
else
    WETH_NAME_TEST=$(cast call $WETH_ADDRESS "name()(string)" --rpc-url $RPC_URL)
    if [ $WETH_NAME_TEST != "Wrapped Ether" ]; then
        echo "❌ Seems that token name is wrong. Received: $WETH_NAME_TEST"
        exit 1
    else
        echo "✅ WETH Got deployed as intended, at $WETH_ADDRESS"
    fi
fi

# Build Manager
echo "🛠️ Building Manager..."
cargo build --release --target wasm32-unknown-unknown --features manager

# Deploy Manager
echo "🚀 Deploying Manager..."
MANAGER_ADDRESS=$(cargo stylus deploy \
    --private-key=$PRIVATE_KEY \
    --wasm-file target/wasm32-unknown-unknown/release/libmicrostable.wasm \
    --no-verify 2>/dev/null | grep "deployed code at address:" | awk '{print $5}' | tr -d '[:space:]' | tr -d '\r' | sed 's/\x1b\[[0-9;]*m//g')

# Check address length
if [ ${#MANAGER_ADDRESS} -ne 42 ]; then
    echo "Error: MANAGER_ADDRESS has incorrect length: ${#MANAGER_ADDRESS}"
    exit 1
fi

echo "✅ Manager Got deployed as intended, at $MANAGER_ADDRESS"

# Build Test Oracle
echo "🛠️ Building Test Oracle..."
cargo build --release --target wasm32-unknown-unknown --features test-oracle

# Deploy Test Oracle
echo "🚀 Deploying Test Oracle..."
TEST_ORACLE_ADDRESS=$(cargo stylus deploy \
    --private-key=$PRIVATE_KEY \
    --wasm-file target/wasm32-unknown-unknown/release/libmicrostable.wasm \
    --no-verify 2>/dev/null | grep "deployed code at address:" | awk '{print $5}' | tr -d '[:space:]' | tr -d '\r' | sed 's/\x1b\[[0-9;]*m//g')

# Check address length
if [ ${#TEST_ORACLE_ADDRESS} -ne 42 ]; then
    echo "Error: TEST_ORACLE_ADDRESS has incorrect length: ${#TEST_ORACLE_ADDRESS}"
    exit 1
fi

echo "✅ Oracle Got deployed as intended, at $TEST_ORACLE_ADDRESS"
echo "🦀 All contracts deployed, lets move on to setup:"

# Weth setup
echo "🏃 Setting up WETH contract..."
BALANCE_BEFORE=$(cast call $WETH_ADDRESS "balanceOf(address)(uint256)" $PUB_KEY --rpc-url $RPC_URL)
if [ "$BALANCE_BEFORE" -ne "0" ]; then
    echo "❌ Somehow you already had a weth balance??"
    exit 1
fi

echo "🧑‍💼 Set deployer as admin..."
cast send $WETH_ADDRESS "init(address)" $PUB_KEY --rpc-url $RPC_URL --private-key $PRIVATE_KEY

echo "💸 Minting WETH to wallet..."
cast send $WETH_ADDRESS "mint(address, uint256)" $PUB_KEY 1000000000000000000 --rpc-url $RPC_URL --private-key $PRIVATE_KEY

BALANCE_AFTER=$(cast call $WETH_ADDRESS "balanceOf(address)(uint256)" $PUB_KEY --rpc-url $RPC_URL -- --to-dec)
if [ $BALANCE_AFTER != 1000000000000000000 ]; then
    echo "❌ WETH mint to deployer did not work. Balance: $BALANCE_AFTER"
    exit 1
fi

echo "✅ WETH setup completed successfully."

# ShUSD setup
echo "🏃 Setting up ShUSD contract..."
echo "🧑‍💼 Set Manager as admin..."
cast send $SH_USD_ADDRESS "init(address)" $MANAGER_ADDRESS --rpc-url $RPC_URL --private-key $PRIVATE_KEY
echo "✅ DONE"

# Manager Setup
echo "🏃 Setting up Manager contract..."
cast send $MANAGER_ADDRESS "init(address,address,address)" $WETH_ADDRESS $TEST_ORACLE_ADDRESS $SH_USD_ADDRESS --rpc-url $RPC_URL --private-key $PRIVATE_KEY
echo "✅ DONE"

# Approve weth for spending
echo "📝 Approving weth for spending by the manager contract"
cast send $WETH_ADDRESS "approve(address,uint256)" $MANAGER_ADDRESS 1000000000000000000 --rpc-url $RPC_URL --private-key $PRIVATE_KEY

# Check allowance
echo "🤓 Controlling WETH"
APPROVAL_OF_MANAGER=$(cast call $WETH_ADDRESS "allowance(address,address)(uint256)" $PUB_KEY $MANAGER_ADDRESS --rpc-url $RPC_URL -- --to-dec)
if [ $APPROVAL_OF_MANAGER != 1000000000000000000 ]; then
    echo "❌ Faulty allowance, correct: $APPROVAL_OF_MANAGER"
    exit 1
fi
echo "✅ Its good!"


echo "🏃 Checking weth balance of manager contract..."
MANAGER_WETH_BALANCE_BEFORE=$(cast call $WETH_ADDRESS "balanceOf(address)(uint256)" $MANAGER_ADDRESS --rpc-url $RPC_URL)
if [ "$MANAGER_WETH_BALANCE_BEFORE" -ne "0" ]; then
    echo "❌ Somehow manager already had a weth balance?? $MANAGER_WETH_BALANCE_BEFORE"
    exit 1
fi

# Deposit weth into contract
echo "📝 Depositing weth into contract"
cast send $MANAGER_ADDRESS "deposit(uint256)" 100000000000000000 --rpc-url $RPC_URL --private-key $PRIVATE_KEY

echo "🏃 Checking weth balance of manager contract, after deposit..."
MANAGER_WETH_BALANCE_AFTER=$(cast call $WETH_ADDRESS "balanceOf(address)(uint256)" $MANAGER_ADDRESS --rpc-url $RPC_URL)
echo "🧐 Manager weth balance is: $MANAGER_WETH_BALANCE_AFTER"
if [ $MANAGER_WETH_BALANCE_AFTER != 100000000000000000 ]; then
    echo "❌ Didnt manage to deposit weth?? Managers balance is $MANAGER_WETH_BALANCE_AFTER"
    exit 1
fi
echo "✅ Deposit success!!"

echo "🏃 Fetching collateral ratio..."
BEFORE_COLLATERAL_RATIO=$(cast call $MANAGER_ADDRESS "collatRatio(address)(uint256)" $PUB_KEY --rpc-url $RPC_URL --private-key $PRIVATE_KEY)
echo "🧐 Collateral ratio is is: $BEFORE_COLLATERAL_RATIO"

echo "🏃 Checking sh usd balance of my wallet..."
MANAGER_SH_USD_BALANCE_BEFORE=$(cast call $SH_USD_ADDRESS "balanceOf(address)(uint256)" $PUB_KEY --rpc-url $RPC_URL)
if [ "$MANAGER_SH_USD_BALANCE_BEFORE" -ne "0" ]; then
    echo "❌ Somehow manager already had a weth balance?? $MANAGER_SH_USD_BALANCE_BEFORE"
    exit 1
fi

echo "🫣 Finally minting, scary, LETS GO!!"

cast send $MANAGER_ADDRESS "mint(uint256)" 10000000000000000000 --rpc-url $RPC_URL --private-key $PRIVATE_KEY
