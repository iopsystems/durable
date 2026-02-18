---
name: format-code
description: Formats the codebase using nightly rustfmt. Use when asked to "format the code", "run rustfmt", "cargo fmt", or when formatting is needed after code changes.
context: root
user-invocable: true
---

# Format Code

## Description
Formats all Rust code in the codebase using nightly rustfmt to ensure consistent code style. This skill always uses nightly rustfmt to access the latest formatting features and configurations.

## When to Use
- When asked to "format the code" or "format my code"
- When asked to "run rustfmt" or "cargo fmt"
- When asked to "fix formatting" or "clean up formatting"
- After completing code changes that need formatting
- Before committing changes to ensure consistent style
- When formatting issues are flagged by CI or pre-commit hooks

## How It Works

### Step 1: Run Nightly Rustfmt
Execute the format command using nightly toolchain:

```bash
cargo +nightly fmt
```

This formats all Rust files in the workspace according to the project's `rustfmt.toml` configuration (if present).

### Step 2: Report Results
After formatting completes:
- If changes were made, inform the user that files were formatted
- If no changes were needed, confirm the code was already properly formatted
- Report any errors that occurred during formatting

## Example Usage

**User Request:** "Format the code"

**Expected Behavior:**

1. **Run formatter**:
   ```bash
   cargo +nightly fmt
   ```

2. **Check for changes** (optional):
   ```bash
   git status
   ```

3. **Report**:
   ```
   Code formatted successfully using nightly rustfmt.
   ```

**User Request:** "Run rustfmt before I commit"

**Expected Behavior:**

1. **Run formatter**:
   ```bash
   cargo +nightly fmt
   ```

2. **Report**:
   ```
   Formatting complete. Ready to commit.
   ```

## Best Practices

### Do:
- Always use `cargo +nightly fmt` to ensure nightly toolchain
- Run formatting before committing changes
- Report whether any files were modified by the formatter

### Don't:
- Use `cargo fmt` without the `+nightly` flag
- Skip formatting when making code changes
- Ignore formatting errors

## Quality Checklist

Before considering the task complete, verify:
- [ ] Ran `cargo +nightly fmt` successfully
- [ ] No formatting errors were reported
- [ ] User is informed of the result

## Troubleshooting

**Nightly toolchain not installed:**
- Run `rustup install nightly` to install
- Then retry the format command

**Formatting fails with parse errors:**
- Check for syntax errors in the Rust code
- Fix compilation errors first, then format

**rustfmt.toml issues:**
- Verify the configuration file is valid
- Check that nightly-only options are supported

## Related Skills
- `create-git-commit` - Create commits after formatting
- Code quality and linting workflows
