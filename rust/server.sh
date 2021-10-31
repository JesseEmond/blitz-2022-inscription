#!/usr/bin/env bash

TETROMINOS=(
	I
	J
	L
	O
	S
	T
	Z
)

BASE_JSON='{"tick": 0, "payload": {"totems": []}}'

function generate_problem() {
	local size="$1" shape
	local json="$BASE_JSON"
	while [ "$size" -gt 0 ]; do
		shape=$((RANDOM % ${#TETROMINOS[@]}))
		json="$(
			jq -c --arg shape "${TETROMINOS[$shape]}" \
				'.payload.totems |= . + [{"shape": $shape}]' \
				<<< "$json"
		)"
		size=$((size - 1))
	done
	printf '%s\n' "$json"
}

function server_loop() {
	local line msg_type token actions size=1
	while IFS= read -r line; do
		msg_type="$(jq -r '.type' <<< "$line")"
		case "$msg_type" in
			REGISTER)
				token="$(jq -r '.token' <<< "$line")"
				printf 'Accept client with token %s\n' "$token" >&2
				generate_problem $size
				;;
			COMMAND)
				actions="$(jq -r '.actions' <<< "$line")"
				size=$((size * 2))
				if [ "$size" -gt 32 ]; then
					exit
				fi
				generate_problem $size
				;;
			*)
				break
				;;
		esac
	done
}

server_loop
