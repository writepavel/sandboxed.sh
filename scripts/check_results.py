#!/usr/bin/env python3
"""Check task results from the model comparison test."""

import json
import requests
import sys
import os

API_URL = "https://agent-backend.thomas.md"

# Task IDs from the test (round 2 - with fixed default model)
TASKS = {
    "moonshotai/kimi-k2-thinking": "108bfe55-e937-4ff4-b71e-5865370c8191",
    "x-ai/grok-4.1-fast": "856703ff-f5d1-401d-9f3b-e7f965e4524d",
    "deepseek/deepseek-v3.2-speciale": "a404d71d-f22c-4c38-ac18-7332e39c8b6b",
    "mistralai/mistral-large-2512": "87972676-e4cf-4b23-8f8e-1043169bc12d",
    "anthropic/claude-sonnet-4.5": "e2e1bb84-aaab-410a-b133-68a182901576",
}


def get_token():
    """Get auth token."""
    # Try to get password from secrets.json
    secrets_path = os.path.join(os.path.dirname(__file__), "..", "secrets.json")
    password = ""
    if os.path.exists(secrets_path):
        with open(secrets_path) as f:
            secrets = json.load(f)
            # Try different possible keys
            password = (
                secrets.get("dashboard_password") or 
                secrets.get("dashboard", {}).get("password") or
                secrets.get("auth", {}).get("dashboard_password") or
                ""
            )
    if not password:
        password = os.environ.get("DASHBOARD_PASSWORD", "")
    
    if not password:
        print("Error: No dashboard password found")
        sys.exit(1)
    
    resp = requests.post(f"{API_URL}/api/auth/login", json={"password": password})
    data = resp.json()
    return data.get("token")


def check_task(token, model, task_id):
    """Check a task's status."""
    headers = {"Authorization": f"Bearer {token}"}
    try:
        resp = requests.get(f"{API_URL}/api/task/{task_id}", headers=headers)
        data = resp.json()
        return {
            "model": model,
            "task_id": task_id,
            "status": data.get("status", "unknown"),
            "iterations": data.get("iterations", 0),
            "result_length": len(data.get("result", "")),
            "result_preview": data.get("result", "")[:200],
            "error": "Error:" in data.get("result", ""),
        }
    except Exception as e:
        return {
            "model": model,
            "task_id": task_id,
            "status": "error",
            "iterations": 0,
            "result_length": 0,
            "result_preview": str(e),
            "error": True,
        }


def main():
    token = get_token()
    if not token:
        print("Failed to get auth token")
        sys.exit(1)
    
    print("=" * 80)
    print("Quick Model Test Results")
    print("=" * 80)
    print()
    
    results = []
    for model, task_id in TASKS.items():
        result = check_task(token, model, task_id)
        results.append(result)
    
    # Print summary table
    print(f"{'Model':<45} | {'Status':<10} | {'Iters':<5} | {'Chars':<8} | {'Error'}")
    print("-" * 45 + "-+-" + "-" * 10 + "-+-" + "-" * 5 + "-+-" + "-" * 8 + "-+-------")
    
    for r in results:
        error_mark = "❌" if r["error"] else "✓"
        print(f"{r['model']:<45} | {r['status']:<10} | {r['iterations']:<5} | {r['result_length']:<8} | {error_mark}")
    
    print()
    print("=" * 80)
    print("Detailed Results")
    print("=" * 80)
    
    # Categorize results
    working = [r for r in results if r["status"] == "completed" and not r["error"]]
    failed = [r for r in results if r["status"] == "failed" or r["error"]]
    running = [r for r in results if r["status"] in ("pending", "running")]
    
    print(f"\n✓ Working models ({len(working)}):")
    for r in working:
        print(f"  - {r['model']}: {r['result_preview'][:100]}...")
    
    print(f"\n❌ Failed models ({len(failed)}):")
    for r in failed:
        print(f"  - {r['model']}: {r['result_preview'][:150]}...")
    
    if running:
        print(f"\n⏳ Still running ({len(running)}):")
        for r in running:
            print(f"  - {r['model']}")
    
    # Summary
    print()
    print("=" * 80)
    print("SUMMARY")
    print("=" * 80)
    print(f"Working: {len(working)}/{len(results)}")
    print(f"Failed: {len(failed)}/{len(results)}")
    print(f"Running: {len(running)}/{len(results)}")
    
    return results


if __name__ == "__main__":
    main()
