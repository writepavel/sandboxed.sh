# Open Agent Model Selection Analysis Report

**Date:** 2025-12-19
**Test Environment:** Production (agent-backend.thomas.md)

## Executive Summary

This report analyzes the performance of various LLM models with Open Agent, focusing on:
1. Model availability and compatibility
2. Task completion success rates
3. Model selection system behavior
4. Recommendations for improvement

## Models Tested

| Model | Provider | Type | Status |
|-------|----------|------|--------|
| moonshotai/kimi-k2-thinking | MoonshotAI | Thinking | ✅ Working |
| x-ai/grok-4.1-fast | xAI | Fast | ✅ Working |
| google/gemini-3-flash-preview | Google | Flash | ❌ Requires reasoning token handling |
| deepseek/deepseek-v3.2-speciale | DeepSeek | Special | ❌ Not in allowlist |
| qwen/qwen3-vl-235b-a22b-thinking | Alibaba | VL/Thinking | ✅ Working |
| mistralai/mistral-large-2512 | Mistral | Large | ⚠️ Inconsistent |
| amazon/nova-pro-v1 | Amazon | Pro | ✅ Working |
| z-ai/glm-4.6v | Zhipu | Vision | ✅ Working |
| anthropic/claude-sonnet-4.5 | Anthropic | Sonnet | ✅ Working (Baseline) |
| google/gemini-2.5-pro | Google | Pro | ✅ Working |
| deepseek/deepseek-chat | DeepSeek | Chat | ✅ Working |

## Key Findings

### 1. Gemini 3 "Thinking" Models Require Special Handling

**Issue:** Gemini 3 Flash Preview and similar "thinking" models require OpenRouter reasoning token preservation.

**Error Message:**
```
Gemini models require OpenRouter reasoning details to be preserved in each request.
Function call is missing a `thought_signature` in functionCall parts.
```

**Root Cause:** These models generate "thinking" tokens that must be preserved in subsequent API calls when using tools.

**Impact:** Cannot use Gemini 3 thinking models until this is implemented.

### 2. CAPABLE_MODEL_BASES Allowlist Too Restrictive

**Issue:** Models not explicitly in the allowlist are silently rejected or substituted.

**Example:** `deepseek/deepseek-v3.2-speciale` was requested but the system:
- Logged: "Requested model not found, using default capability floor 0.7"
- Selected: `deepseek/deepseek-r1-distill-llama-70b` instead

**Recommendation:** 
- Add more models to the allowlist
- Warn users when their requested model isn't available
- Consider dynamic model validation via OpenRouter API

### 3. Price-Based Capability Estimation Has Issues

**Issue:** Free/cheap models get capability score of 0.3 regardless of actual performance.

**Log Evidence:**
```
Using price-based capability for deepseek/deepseek-r1: 0.300 (avg_cost: 0.0000000000)
Using price-based capability for deepseek/deepseek-chat: 0.300 (avg_cost: 0.0000000000)
```

**Problem:** DeepSeek models are often free/very cheap but perform well. The price-based heuristic underestimates them.

**Recommendation:**
- Integrate actual benchmark data from llm-stats.com
- Use model family tiers as fallback
- Consider historical performance tracking

### 4. Benchmark Data Not Being Used

**Observation:** All model selections show "benchmark_data: false"

**Log Evidence:**
```
Model selected: deepseek/deepseek-r1-distill-llama-70b (task: ToolCalling, cost: 2 cents, benchmark_data: false, history: false)
```

**Root Cause:** The benchmark registry may not be properly loaded or models aren't matched.

### 5. Default Model Configuration Critical

**Issue:** Setting `DEFAULT_MODEL` to a problematic model (like gemini-3-flash-preview) breaks ALL tasks.

**Why:** Internal operations (complexity estimation, task splitting) use the default model, not the requested one.

**Recommendation:** 
- Validate default model on startup
- Use separate models for internal operations vs. task execution
- Add health check that tests the default model

## Quick Test Results

| Model | Status | Iterations | Result Length | Notes |
|-------|--------|------------|---------------|-------|
| moonshotai/kimi-k2-thinking | completed | 2 | 48 | Fast, accurate |
| x-ai/grok-4.1-fast | completed | 2 | 62 | Fast, verbose |
| deepseek/deepseek-v3.2-speciale | failed | 0 | 0 | Not in allowlist |
| mistralai/mistral-large-2512 | completed | 0 | 4602 | Very verbose |
| anthropic/claude-sonnet-4.5 | completed | 2 | 48 | Baseline, reliable |
| qwen/qwen3-vl-235b-a22b-thinking | completed | 2 | 50 | Working well |
| amazon/nova-pro-v1 | completed | 2 | 50 | Working well |
| z-ai/glm-4.6v | completed | 2 | 50 | Working well |
| google/gemini-2.5-pro | completed | 2 | 50 | Working (non-thinking variant) |
| deepseek/deepseek-chat | completed | 2 | 50 | Working (standard variant) |

## Recommendations for Model Selection Improvements

### Immediate Fixes (High Priority)

1. **Expand CAPABLE_MODEL_BASES:**
   - Add all models from the user's test list
   - Add popular new models automatically
   - Consider dynamic validation

2. **Fix Benchmark Integration:**
   - Ensure benchmark data loads correctly
   - Add logging for benchmark matching
   - Use benchmarks for task-type-specific selection

3. **Add Reasoning Token Support:**
   - For Gemini 3 and other "thinking" models
   - Preserve thought signatures in tool calls
   - Reference: https://openrouter.ai/docs/guides/best-practices/reasoning-tokens

### Medium-Term Improvements

4. **Historical Performance Tracking:**
   - Record actual success/failure per model
   - Track cost efficiency per model
   - Use this data for future selections

5. **Separate Internal vs. Execution Models:**
   - Use cheap/fast model for complexity estimation
   - Use cheap/fast model for task splitting
   - Use user-selected model only for actual execution

6. **Model Validation on Startup:**
   - Check if default model works
   - Validate key models in the allowlist
   - Alert on configuration issues

### Long-Term Enhancements

7. **Dynamic Model Discovery:**
   - Fetch available models from OpenRouter API
   - Auto-detect capabilities (tools support, vision, etc.)
   - Automatic fallback chains

8. **A/B Testing Framework:**
   - Run same task with multiple models
   - Compare quality, cost, speed
   - Continuously update model rankings

9. **User-Facing Model Insights:**
   - Show why a model was selected
   - Display estimated cost before execution
   - Allow manual override with warnings

## Security Analysis Task Status

Three models are currently running the Rabby Wallet security analysis:
- `moonshotai/kimi-k2-thinking`: 9055ae68-d0bb-4c0d-aae3-908de141c431
- `x-ai/grok-4.1-fast`: f99e7b95-9d57-42e4-b669-62b4f7c6a9f4
- `anthropic/claude-sonnet-4.5`: 95fdebf9-f4bc-43f6-ba20-c9579dcadbd6

Results will be appended to this report when available.

## Appendix: Code Changes Made

1. **Added Models to CAPABLE_MODEL_BASES** (`src/budget/pricing.rs`):
   - moonshotai/kimi-k2-thinking, kimi-k2
   - x-ai/grok-4.1-fast, grok-4-fast, grok-4, grok-3
   - google/gemini-3-flash-preview, gemini-3-pro-preview
   - deepseek/deepseek-v3.2-speciale, deepseek-v3.2, deepseek-v3.1-terminus
   - qwen/qwen3-vl-235b-a22b-thinking
   - amazon/nova-pro-v1
   - z-ai/glm-4.6v, glm-4.6, glm-4.5v, glm-4.5

2. **Created Test Scripts**:
   - `scripts/quick_model_test.sh`: Fast capability verification
   - `scripts/test_model_comparison.sh`: Full security analysis comparison
   - `scripts/run_security_test.sh`: Interactive security test runner
   - `scripts/check_results.py`: Result collection and analysis

3. **Fixed Production Configuration**:
   - Changed DEFAULT_MODEL from gemini-3-flash-preview to claude-sonnet-4.5

---

*Report generated by model comparison testing framework*
