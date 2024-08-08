#!/bin/bash

# Number of iterations
NUM_ITERATIONS=10

# Initialize arrays to store timings
RUST_TIMES=()
PYTHON_TIMES=()

# Function to extract seconds from timing output
extract_seconds() {
    echo "$1" | awk '{print $2}' | sed 's/m//;s/s//'
}

# Run the Rust CLI tool NUM_ITERATIONS times
echo "Running Rust CLI tool in release mode $NUM_ITERATIONS times..."
timer_start=$(($(date +%s%N)/1000000))
for ((i = 1; i <= NUM_ITERATIONS; i++)); do
    RUST_OUTPUT=$( (time release/windows/tomatwo.exe --input food-test.avi) 2>&1 )
    RUST_TIME=$(echo "$RUST_OUTPUT" | grep "real" | extract_seconds)
    RUST_TIMES+=("$RUST_TIME")
done
timer_end=$(($(date +%s%N)/1000000))
timer_diff=$((timer_end - timer_start))
echo "Rust CLI loop took $timer_diff ms to run $NUM_ITERATIONS times"

# Run the Python script NUM_ITERATIONS times
echo "Running Python script $NUM_ITERATIONS times..."
timer_start=$(($(date +%s%N)/1000000))
for ((i = 1; i <= NUM_ITERATIONS; i++)); do
    PYTHON_OUTPUT=$( (time python tomato-py/tomato.py --input food-test.avi) 2>&1 )
    PYTHON_TIME=$(echo "$PYTHON_OUTPUT" | grep "real" | extract_seconds)
    PYTHON_TIMES+=("$PYTHON_TIME")
done
timer_end=$(($(date +%s%N)/1000000))
timer_diff=$((timer_end - timer_start))
echo "Python script loop took $timer_diff ms to run $NUM_ITERATIONS times"

# Function to calculate average using awk
average() {
    local sum=0
    local count=$#
    for num in "$@"; do
        sum=$(echo "$sum + $num" | awk '{printf "%.6f", $1}')
    done
    echo "scale=6; $sum / $count" | awk '{printf "%.6f", $1}'
}

# Function to calculate min using awk
min() {
    echo "$@" | tr ' ' '\n' | awk 'NR==1 {min=$1} $1 < min {min=$1} END {printf "%.6f", min}'
}

# Function to calculate max using awk
max() {
    echo "$@" | tr ' ' '\n' | awk 'NR==1 {max=$1} $1 > max {max=$1} END {printf "%.6f", max}'
}

# Function to calculate standard deviation using awk
stddev() {
    local avg=$(average "$@")
    local sum=0
    local count=$#
    for num in "$@"; do
        local diff=$(echo "$num - $avg" | awk '{printf "%.6f", $1}')
        local sqr=$(echo "$diff * $diff" | awk '{printf "%.6f", $1}')
        sum=$(echo "$sum + $sqr" | awk '{printf "%.6f", $1}')
    done
    local variance=$(echo "scale=6; $sum / $count" | awk '{printf "%.6f", $1}')
    echo "scale=6; sqrt($variance)" | awk '{printf "%.6f", sqrt($1)}'
}

# Calculate and display statistics for Rust timings
RUST_AVG=$(average "${RUST_TIMES[@]}")
RUST_MIN=$(min "${RUST_TIMES[@]}")
RUST_MAX=$(max "${RUST_TIMES[@]}")
RUST_STDDEV=$(stddev "${RUST_TIMES[@]}")

echo "Rust timings (seconds):"
echo "Average: $RUST_AVG"
echo "Min: $RUST_MIN"
echo "Max: $RUST_MAX"
echo "Standard Deviation: $RUST_STDDEV"

# Calculate and display statistics for Python timings
PYTHON_AVG=$(average "${PYTHON_TIMES[@]}")
PYTHON_MIN=$(min "${PYTHON_TIMES[@]}")
PYTHON_MAX=$(max "${PYTHON_TIMES[@]}")
PYTHON_STDDEV=$(stddev "${PYTHON_TIMES[@]}")

echo "Python timings (seconds):"
echo "Average: $PYTHON_AVG"
echo "Min: $PYTHON_MIN"
echo "Max: $PYTHON_MAX"
echo "Standard Deviation: $PYTHON_STDDEV"
