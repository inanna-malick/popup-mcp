#!/bin/bash

echo "Testing enhanced error messages..."
echo

# Test 1: Missing quotes
echo "Test 1: Missing quotes around title"
echo 'popup Title []' | cargo run --bin stdio_direct 2>&1 | grep -A 10 "Parse error"
echo

# Test 2: Invalid widget
echo "Test 2: Invalid widget in simplified syntax"
echo '[Title: invalid_widget "test"]' | cargo run --bin stdio_direct 2>&1 | grep -A 10 "Parse error" 
echo

# Test 3: Empty popup
echo "Test 3: Empty popup body"
echo 'popup "Bad" [' | cargo run --bin stdio_direct 2>&1 | grep -A 10 "Parse error"
echo

echo "Done!"