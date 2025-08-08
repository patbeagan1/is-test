#!/bin/zsh

# Integration test script for the 'is-test' command.

# Exit immediately if a command exits with a non-zero status.
set -o pipefail

# --- Test Runner Setup ---

# Build the project before running tests
echo "Building the project..."
/bin/env cargo build --quiet

# The path to the compiled binary
readonly IS_CMD="./target/debug/is-test"

# Counter for the number of tests run
test_count=0
# Counter for the number of tests that passed
pass_count=0

# Function to run a test
#
# Usage:
#   test_case <description> <command_to_run>
#
# Example:
#   test_case "File should exist" "$IS_CMD file exists README.md"

# Define color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

function test_case() {
    local description="$1"
    local command_to_run="$2"
    
    test_count=$((test_count + 1))
    echo -n "${YELLOW}Test $test_count: $description... ${NC}"
    
    # Execute the command and capture the exit code
    eval "$command_to_run"
    local exit_code=$?
    
    # Check if the exit code is 0 (success)
    if [[ $exit_code -eq 0 ]]; then
        echo "${GREEN}PASS${NC}"
        pass_count=$((pass_count + 1))
    else
        echo "${RED}FAIL${NC} (exit code: $exit_code)"
    fi
}

function test_case_fails() {
    local description="$1"
    local command_to_run="$2"
    
    ((test_count++))
    echo -n "${YELLOW}Test $test_count: $description... ${NC}"
    
    # Execute the command and capture the exit code
    eval "$command_to_run"
    local exit_code=$?
    
    # Check if the exit code is not 0 (failure)
    if [[ $exit_code -ne 0 ]]; then
        echo "${GREEN}PASS${NC}"
        ((pass_count++))
    else
        echo "${RED}FAIL${NC} (expected non-zero exit code)"
    fi
}


# --- Test Fixture Setup ---

# Create dummy files and directories for testing
readonly TEST_DIR="test_fixtures"
rm -rf "$TEST_DIR"
mkdir -p "$TEST_DIR"
touch "$TEST_DIR/empty_file.txt"
echo "hello" > "$TEST_DIR/file.txt"
mkdir "$TEST_DIR/subdir"
ln -s "$TEST_DIR/file.txt" "$TEST_DIR/symlink_to_file"
mkfifo "$TEST_DIR/named_pipe"

# --- Test Cases ---

########################################################

echo "\n--- Running File Tests ---"

alias is="$IS_CMD"

test_case "File exists" \
          "is file exists $TEST_DIR/file.txt"
          
test_case_fails "File does not exist" \
          "is file exists $TEST_DIR/nonexistent.txt"
          
test_case "Is a directory" \
          "is file directory $TEST_DIR/subdir"
          
test_case_fails "Is not a directory" \
          "is file directory $TEST_DIR/file.txt"
          
test_case "Is a regular file" \
          "is file regular $TEST_DIR/file.txt"
          
test_case "Is a symlink" \
          "is file symlink $TEST_DIR/symlink_to_file"
          
test_case "Is a named pipe" \
          "is file named-pipe $TEST_DIR/named_pipe"
          
test_case "File is not empty" \
          "is file non-empty $TEST_DIR/file.txt"
          
test_case_fails "File is empty" \
          "is file non-empty $TEST_DIR/empty_file.txt"
          
test_case "File is readable" \
          "is file readable $TEST_DIR/file.txt"
          
test_case "File is writable" \
          "is file writable $TEST_DIR/file.txt"
          
########################################################

echo "\n--- Running String Tests ---"

test_case "Strings are equal" \
          "is string equal 'hello' 'hello'"

test_case_fails "Strings are not equal" \
          "is string equal 'hello' 'world'"

test_case "Strings are not equal" \
          "is string not-equals 'hello' 'world'"

test_case "String is empty" \
          "is string empty ''"

test_case "String is not empty" \
          "is string not-empty 'hello'"

test_case "String equals (case-insensitive)" \
          "is string equal-ci 'Hello' 'hello'"

test_case "String matches regex" \
          "is string matches-regex 'hello' '^h.l.o$'"

test_case_fails "String does not match regex" \
          "is string matches-regex 'world' '^h.l.o$'"

test_case "String contains" \
          "is string contains 'hello world' 'lo w'"

test_case "String starts with" \
          "is string starts-with 'hello' 'he'"

test_case "String ends with" \
          "is string ends-with 'hello' 'lo'"


########################################################

echo "\n--- Running Integer Tests ---"

test_case "Numbers are equal" \
          "is int eq 10 10"
          
test_case_fails "Numbers are not equal" \
          "is int eq 10 5"
          
test_case "Number is greater than" \
          "is int gt 10 5"
          
test_case "Number is greater than or equal" \
          "is int ge 10 10"
          
test_case "Number is less than" \
          "is int lt 5 10"
          
test_case "Number is less than or equal" \
          "is int le 10 10"
          
test_case "Number in range" \
          "is int in-range 7 5 10"
          
test_case_fails "Number not in range" \
          "is int in-range 12 5 10"
          

########################################################

echo "\n--- Running Float Tests ---"

test_case "Floats are equal" \
          "is float eq 10.5 10.5"
          
test_case "Float is greater than" \
          "is float gt 10.5 5.5"
          
test_case "Float approximately equal" \
          "is float approx-eq 10.0 10.0001 0.001"
          
test_case_fails "Float not approximately equal" \
          "is float approx-eq 10.0 10.1 0.001"
          

########################################################

echo "\n--- Running Semver Tests ---"

test_case "Semver is equal" \
          "is semver eq 1.2.3 1.2.3"
          
test_case "Semver is greater than" \
          "is semver gt 1.2.4 1.2.3"
          
test_case "Semver is less than" \
          "is semver lt 1.2.3 1.2.4"
          

########################################################

echo "\n--- Running Env Tests ---"

test_case "Env var is set" \
          "TEST_VAR=hello is env set TEST_VAR"

test_case_fails "Env var is not set" \
          "is env set NON_EXISTENT_VAR"
          
test_case "Env var equals value" "TEST_VAR=hello is env equal-to TEST_VAR hello"

########################################################

echo "\n--- Running System Tests ---"

test_case "OS is linux" \
          "is system os linux"
          
test_case "Command exists" \
          "is system command-exists ls"
          
test_case_fails "Command does not exist" \
          "is system command-exists non_existent_command_12345"
          
test_case "Arch is `uname -m`" \
          "is system arch `uname -m`"
          
########################################################

# testing usage with the if builtin

echo "\n--- Testing usage with the if builtin ---"

if is file regular $TEST_DIR/file.txt; then
    echo "${GREEN}PASS: File is regular: $TEST_DIR/file.txt${NC}"
else
    echo "${RED}FAIL: File is not regular: $TEST_DIR/file.txt${NC}"
fi

if is file regular $TEST_DIR/nonexistent.txt; then
    echo "${RED}FAIL: File is regular: $TEST_DIR/nonexistent.txt${NC}"
else
    echo "${GREEN}PASS: File is not regular: $TEST_DIR/nonexistent.txt${NC}"
fi

if is system os linux; then
    echo "${GREEN}PASS: OS is linux${NC}"
else
    echo "${RED}FAIL: OS is not linux${NC}"
fi

# if is connected to the internet
if is net online; then
    echo "${GREEN}PASS: Connected to internet${NC}"
else
    echo "${RED}FAIL: Not connected to internet${NC}"
fi

# --- Test Cleanup ---
rm -rf "$TEST_DIR"

# --- Test Summary ---
echo "\n--- Test Summary ---"
echo "Total tests: $test_count"
echo "Passed:      $pass_count"
if [[ $pass_count -ne $test_count ]]; then
    echo "Some tests failed!"
    exit 1
else
    echo "All tests passed!"
    exit 0
fi