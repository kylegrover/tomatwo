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
        print(f"  Run {i+1}: {time_taken:.4f}s")
    print(f"{name} completed {len(times)} runs.\n")
    return times

def print_statistics(rust_times, python_times):
    if not rust_times or not python_times:
        return

    rust_avg = statistics.mean(rust_times)
    rust_min = min(rust_times)
    rust_max = max(rust_times)
    python_avg = statistics.mean(python_times)
    python_min = min(python_times)
    python_max = max(python_times)

    max_time = max(rust_max, python_max)
    scale = 40 / max_time  # Scale factor for 40 character width

    print("Comparative Timings (seconds):")

    def print_bar(name, rust_val, python_val):
        rust_bar = int(rust_val * scale)
        python_bar = int(python_val * scale)
        print(f"{name:<7}");
        # print(f"   {'█' * rust_bar:<40} {rust_val:.6f}")
        # print(f"           {'█' * python_bar:<40} {python_val:.6f}")
        print(f"Rust:    {'❙' * rust_bar:<40} {rust_val:.6f}")   
        print(f"Python:  {'❙' * python_bar:<40} {python_val:.6f}")

    print_bar("Average", rust_avg, python_avg)
    print_bar("Min", rust_min, python_min)
    print_bar("Max", rust_max, python_max)

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

# Print statistics with visualization
print_statistics(rust_times, python_times)

# Compare results
compare_results(rust_times, python_times)