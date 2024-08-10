import subprocess
import time
import statistics
import os

# Number of iterations
NUM_ITERATIONS = 10

# Paths to executables and input file
RUST_EXECUTABLE = os.path.join("release", "windows", "tomatwo.exe")
PYTHON_SCRIPT = os.path.join("tomato-py", "tomato.py")
INPUT_FILE = "food-test.avi"

# Function to measure execution time
def measure_time(command):
    start_time = time.time()
    process = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, shell=True, text=True, encoding='utf-8')
    end_time = time.time()
    
    if process.returncode != 0:
        print(f"Error: {process.stderr}")
        return None
    
    return end_time - start_time

# Measure Rust performance
rust_times = []
print(f"Running Rust CLI tool {NUM_ITERATIONS} times...")

for _ in range(NUM_ITERATIONS):
    time_taken = measure_time(f"{RUST_EXECUTABLE} --input {INPUT_FILE}")
    if time_taken is not None:
        rust_times.append(time_taken)

print(f"Rust CLI tool completed {NUM_ITERATIONS} runs.")

# Measure Python performance
python_times = []
print(f"Running Python script {NUM_ITERATIONS} times...")

for _ in range(NUM_ITERATIONS):
    time_taken = measure_time(f"python {PYTHON_SCRIPT} --input {INPUT_FILE}")
    if time_taken is not None:
        python_times.append(time_taken)

print(f"Python script completed {NUM_ITERATIONS} runs.")

# Function to print statistics
def print_statistics(name, times):
    if times:
        print(f"{name} timings (seconds):")
        print(f"  Average: {statistics.mean(times):.6f}")
        print(f"  Min: {min(times):.6f}")
        print(f"  Max: {max(times):.6f}")
        print(f"  Standard Deviation: {statistics.stdev(times):.6f}")


# Print statistics for Rust and Python timings
print_statistics("Rust", rust_times)
print_statistics("Python", python_times)