import subprocess
import time
import statistics
import os
import sys

# Number of iterations
NUM_ITERATIONS = 10

# Paths to executables and input file
RUST_EXECUTABLE = os.path.join("release", "windows", "tomatwo.exe")
PYTHON_SCRIPT = os.path.join("tomato-py", "tomato.py")
INPUT_FILE = "food-test.avi"

def measure_time(command):
    start_time = time.time()
    process = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, shell=False)
    end_time = time.time()
    
    if process.returncode != 0:
        print(f"Error: {process.stderr.decode('utf-8', errors='replace')}")
        return None
    
    return end_time - start_time

def run_tests(command, name):
    print(f"Running {name} {NUM_ITERATIONS} times...")
    times = []
    for i in range(NUM_ITERATIONS):
        time_taken = measure_time(command)
        if time_taken is not None:
            times.append(time_taken)
        print(f"  Run {i+1}: {'.' * int(time_taken * 100)} {time_taken:.4f}s")
    print(f"{name} completed {len(times)} runs.\n")
    return times

def print_statistics(name, times):
    if times:
        avg = statistics.mean(times)
        min_time = min(times)
        max_time = max(times)
        std_dev = statistics.stdev(times)
        
        print(f"{name} timings (seconds):")
        print(f"  Average: {avg:.6f}")
        print(f"  Min:     {min_time:.6f}")
        print(f"  Max:     {max_time:.6f}")
        print(f"  Std Dev: {std_dev:.6f}")
        
        # Simple ASCII visualization
        print("\n  Distribution:")
        for t in times:
            print(f"  {'|' * int((t - min_time) / (max_time - min_time) * 20):20s} {t:.6f}")
        print()

def compare_results(rust_times, python_times):
    if rust_times and python_times:
        rust_avg = statistics.mean(rust_times)
        python_avg = statistics.mean(python_times)
        speedup = python_avg / rust_avg
        print(f"Comparison:")
        print(f"  Rust is {speedup:.2f}x faster than Python on average")

# Run tests
rust_command = [RUST_EXECUTABLE, "--input", INPUT_FILE]
python_command = [sys.executable, PYTHON_SCRIPT, "--input", INPUT_FILE]

rust_times = run_tests(rust_command, "Rust CLI tool")
python_times = run_tests(python_command, "Python script")

# Print statistics
print_statistics("Rust", rust_times)
print_statistics("Python", python_times)

# Compare results
compare_results(rust_times, python_times)