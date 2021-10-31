#!/usr/bin/env sh

websocat \
	--text \
	--exit-on-eof \
	ws-l:127.0.0.1:8765 \
	sh-c:./server.sh
