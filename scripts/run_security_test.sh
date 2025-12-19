#!/bin/bash
# Run the Rabby Wallet security analysis with multiple models
# This script submits the task to each model and monitors progress

set -e

API_URL="https://agent-backend.thomas.md"
RESULTS_DIR="$(dirname "$0")/../test_results/security_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$RESULTS_DIR"

echo "==========================================="
echo "Rabby Wallet Security Analysis - Model Comparison"
echo "Results: $RESULTS_DIR"
echo "==========================================="

# Models to test (prioritized)
MODELS=(
    "moonshotai/kimi-k2-thinking"
    "x-ai/grok-4.1-fast"
    "google/gemini-3-flash-preview"
    "deepseek/deepseek-v3.2-speciale"
    "anthropic/claude-sonnet-4.5"  # baseline
)

# The security analysis task
TASK='Download Rabby Wallet extension for Chrome, decompile it, and look for security vulnerabilities similar to the Permit2 transaction simulation bypass bug.

Context on the vulnerability pattern to look for:
- Rabby simulation fails to detect malicious Permit2 approval patterns
- The simulation shows a harmless transaction (e.g., spending 1 USDC) while the actual tx enables draining the user full balance
- The key issue is that the simulation engine does not correctly model Permit2 delegation or spending flows
- The "spender" field from a permit2 should be validated against known safe contract addresses

Focus areas:
1. How Rabby parses and validates Permit2 signatures
2. Whether the spender field is properly validated against known contract addresses
3. If the witness data can be manipulated to display incorrect transaction details
4. Any other transaction simulation bypass vectors

Steps:
1. Download the Rabby extension (https://rabby.io or Chrome Web Store)
2. Extract and decompile the JavaScript code
3. Search for Permit2-related code paths
4. Analyze the simulation/preview logic
5. Identify potential bypass vectors

Provide findings in a structured markdown report with:
- Vulnerability title
- Severity (Critical/High/Medium/Low)
- Description
- Affected code snippets
- Proof of concept outline
- Recommended fix'

# Get auth token
DASHBOARD_PASSWORD="${DASHBOARD_PASSWORD:-}"
if [ -z "$DASHBOARD_PASSWORD" ]; then
    # Try to get from secrets.json
    if [ -f "$(dirname "$0")/../secrets.json" ]; then
        DASHBOARD_PASSWORD=$(jq -r '.dashboard_password // empty' "$(dirname "$0")/../secrets.json")
    fi
fi

if [ -z "$DASHBOARD_PASSWORD" ]; then
    echo "Error: DASHBOARD_PASSWORD not set"
    exit 1
fi

TOKEN=$(curl -s -X POST "$API_URL/api/auth/login" \
    -H "Content-Type: application/json" \
    -d "{\"password\": \"$DASHBOARD_PASSWORD\"}" | jq -r '.token')

if [ -z "$TOKEN" ] || [ "$TOKEN" = "null" ]; then
    echo "Failed to get auth token"
    exit 1
fi

echo "Authenticated successfully"

# Function to submit a task
submit_task() {
    local model="$1"
    local safe_name=$(echo "$model" | tr '/' '_' | tr ':' '_')
    
    echo ""
    echo "Submitting task for: $model"
    
    local payload=$(jq -n \
        --arg task "$TASK" \
        --arg model "$model" \
        '{task: $task, model: $model}')
    
    local response=$(curl -s -X POST "$API_URL/api/task" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $TOKEN" \
        -d "$payload")
    
    local task_id=$(echo "$response" | jq -r '.id // empty')
    
    if [ -z "$task_id" ]; then
        echo "  Failed: $response"
        return 1
    fi
    
    echo "  Task ID: $task_id"
    echo "$task_id" > "$RESULTS_DIR/${safe_name}_task_id.txt"
    
    # Save initial state
    echo "{\"model\": \"$model\", \"task_id\": \"$task_id\", \"submitted_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}" > "$RESULTS_DIR/${safe_name}_meta.json"
}

# Submit all tasks
echo ""
echo "Submitting tasks..."
for model in "${MODELS[@]}"; do
    submit_task "$model"
    sleep 1
done

echo ""
echo "All tasks submitted. Monitoring progress..."
echo "(Press Ctrl+C to stop monitoring)"
echo ""

# Monitor loop
while true; do
    all_done=true
    clear
    echo "==========================================="
    echo "Task Status ($(date))"
    echo "==========================================="
    printf "%-45s | %-10s | %8s | %s\n" "Model" "Status" "Iters" "Result"
    echo "---------------------------------------------+------------+----------+---------"
    
    for model in "${MODELS[@]}"; do
        safe_name=$(echo "$model" | tr '/' '_' | tr ':' '_')
        task_id_file="$RESULTS_DIR/${safe_name}_task_id.txt"
        
        if [ ! -f "$task_id_file" ]; then
            printf "%-45s | %-10s | %8s | %s\n" "$model" "no_task" "-" "-"
            continue
        fi
        
        task_id=$(cat "$task_id_file")
        status_response=$(curl -s "$API_URL/api/task/$task_id" -H "Authorization: Bearer $TOKEN")
        
        status=$(echo "$status_response" | jq -r '.status // "unknown"')
        iterations=$(echo "$status_response" | jq -r '.iterations // 0')
        result_preview=$(echo "$status_response" | jq -r '.result // ""' | head -c 50)
        
        if [ "$status" != "completed" ] && [ "$status" != "failed" ]; then
            all_done=false
        fi
        
        printf "%-45s | %-10s | %8s | %s\n" "$model" "$status" "$iterations" "${result_preview:0:50}"
        
        # Save full result if done
        if [ "$status" = "completed" ] || [ "$status" = "failed" ]; then
            echo "$status_response" | jq . > "$RESULTS_DIR/${safe_name}_result.json"
        fi
    done
    
    if $all_done; then
        echo ""
        echo "All tasks completed!"
        break
    fi
    
    sleep 10
done

# Generate summary
echo ""
echo "==========================================="
echo "Final Summary"
echo "==========================================="

{
    echo "# Model Comparison Results"
    echo ""
    echo "Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo ""
    echo "| Model | Status | Iterations | Result Length | Cost (cents) |"
    echo "|-------|--------|------------|---------------|--------------|"
    
    for model in "${MODELS[@]}"; do
        safe_name=$(echo "$model" | tr '/' '_' | tr ':' '_')
        result_file="$RESULTS_DIR/${safe_name}_result.json"
        
        if [ -f "$result_file" ]; then
            status=$(jq -r '.status' "$result_file")
            iterations=$(jq -r '.iterations' "$result_file")
            result=$(jq -r '.result // ""' "$result_file")
            result_len=${#result}
            # Note: cost would need to be tracked by the agent
            echo "| $model | $status | $iterations | $result_len | - |"
        else
            echo "| $model | no_result | - | - | - |"
        fi
    done
    
    echo ""
    echo "## Detailed Results"
    echo ""
    
    for model in "${MODELS[@]}"; do
        safe_name=$(echo "$model" | tr '/' '_' | tr ':' '_')
        result_file="$RESULTS_DIR/${safe_name}_result.json"
        
        if [ -f "$result_file" ]; then
            echo "### $model"
            echo ""
            jq -r '.result // "No result"' "$result_file"
            echo ""
            echo "---"
            echo ""
        fi
    done
} > "$RESULTS_DIR/REPORT.md"

echo "Report saved to: $RESULTS_DIR/REPORT.md"
echo ""
cat "$RESULTS_DIR/REPORT.md" | head -30
