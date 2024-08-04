#!/bin/bash

# Function to run a command and capture its last line
run_and_capture() {
    local last_line
    last_line=$(eval "$1" | tail -n 1)
    echo "$last_line"
}

# Run Python script
python_output=$(run_and_capture "python tomato.py -i $1")

# Run Rust program
rust_output=$(run_and_capture "cd rust && cargo run --release -- -i ../$1 && cd ..")

# Run Go program
go_output=$(run_and_capture "cd go && go build -ldflags='-s -w' -o tomato && ./tomato -i ../$1 && cd ..")

# Output the results
echo "python: $python_output"
echo "rust: $rust_output"
echo "go: $go_output"