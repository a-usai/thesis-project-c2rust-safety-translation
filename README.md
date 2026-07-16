# thesis-project-c2rust-safety-translation
Thesis project exploring the automated translation from C to Rust. It combines C2Rust with an LLM to autonomously resolve unsafe blocks, featuring custom scripts for warning categorization.

## Requirements

- **Rust**: `1.96.0`
- **Python**: `3.9.6`

---

## Usage

### 1. Generate Clippy report (text)

```bash
cargo clippy > report_clippy.txt 2>&1
```

### 2. Generate Clippy report (JSON)

Analyzes `main.rs` inside `Rust/src/` and produces a structured JSON file:

```bash
cargo clippy --message-format=json 2>&1 | tee clippy_results.json
```

### 3. Run the taxonomy parser

Categorizes Clippy warnings according to the quality taxonomy:

```bash
python3 clippy_parser.py
```

---

## Warning Taxonomy

Warnings are classified according to the two-level, 16-category quality taxonomy from Tadesse et al.:

| Quality Dimension | Category | Description |
|---|---|---|
| Internal Quality | Convention violation | Code that violates common Rust naming and design conventions. |
| Internal Quality | Documentation issues | Issues in comments or documentation that reduce comprehensibility or maintainability. |
| Internal Quality | Inflexible code | Code that uses overly specific types, limiting reusability and flexibility. |
| Internal Quality | Misleading code | Code that leads readers to believe it does something other than what it actually does. |
| Internal Quality | Non-idiomatic code | Code that does not follow Rust conventions, patterns, or best practices. |
| Internal Quality | Non-production code | Code meant for debugging or placeholder purposes that should not appear in production. |
| Internal Quality | Readability issues | Code that makes it harder for readers to understand the developer's intention. |
| Internal Quality | Redundant code | Unnecessarily duplicated code that does not contribute new behavior or logic. |
| External Quality | Arithmetic issues | Patterns that can lead to bugs or undefined behaviors due to arithmetic operations. |
| External Quality | Attribute issues | Improper or missing use of Rust attributes that affect code behavior or stability. |
| External Quality | Compatibility issues | Code that may not work across platforms, Rust versions, or environments. |
| External Quality | Error handling issues | Code that handles errors but hides root causes or limits debuggability. |
| External Quality | Logical issues | Code with valid syntax but likely reflects a misunderstanding in logic. |
| External Quality | Memory safety | Code that risks dangling pointers, buffer overflows, use-after-free, or data races. |
| External Quality | Performance | Code that compiles and runs correctly but leads to inefficient execution. |
| External Quality | Runtime Panic risks | Code that may trigger a panic during execution due to unchecked operations. |
| External Quality | Thread safety | Code that may cause undefined behavior or data races when used across multiple threads. |
| External Quality | Type safety | Code that discards type guarantees. |
