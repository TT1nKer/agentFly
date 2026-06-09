#!/bin/bash

echo "=========================================="
echo "Phase 5: tmux Shell Session Test"
echo "=========================================="

cd "$(dirname "$0")/.."

echo ""
echo -n "  tmux installed... "
if which tmux > /dev/null 2>&1; then
    echo "PASS ($(tmux -V))"
else
    echo "FAIL"
    echo "PHASE 5 FAILED (tmux not installed)"
    exit 1
fi

echo ""
echo "--- tmux Session Lifecycle Test ---"
echo -n "  session created / input sent / output captured / event log / session stopped... "
if cargo test -p agent-bridge --test tmux_shell_test -- --nocapture 2>&1 | grep -q "test result: ok"; then
    echo "PASS"
else
    echo "FAIL"
    echo "PHASE 5 FAILED"
    exit 1
fi

echo ""
echo "=========================================="
echo "tmux installed: PASS"
echo "session created: PASS"
echo "input sent: PASS"
echo "output captured: PASS"
echo "event log written: PASS"
echo "session stopped: PASS"
echo "PHASE 5 PASS"
echo "=========================================="
