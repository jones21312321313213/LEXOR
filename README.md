# 🚀 LEXOR Interpreter Project


**LEXOR** is a strongly typed educational programming language designed to teach Senior High School students the fundamentals of programming. This project is a pure interpreter written entirely in **Rust** from scratch, featuring a hand-written lexer, a recursive-descent parser, and a tree-walking execution engine.

---

## ✨ Features

- **Tree-Walking Interpreter:** Parses source code into an Abstract Syntax Tree (AST) and walks it directly for execution no bytecode, no VM.
- **Hand-Written Lexer:** A dedicated scanning pass tokenizes raw LEXOR source into a clean token stream.
- **Recursive-Descent Parser:** Produces a typed AST of `Stmt` and `Expr` nodes, with clear and descriptive syntax errors.
- **Strong Typing:** Supports `INT`, `FLOAT`, `CHAR`, and strictly quoted `BOOL` (`"TRUE"`/`"FALSE"`) types with full type-checking at runtime.
- **Custom Control Flow:** Enforces LEXOR's unique block structures (`START IF` / `END IF`, `FOR`, `REPEAT WHEN`).
- **Environment / Scope:** A dedicated environment module tracks variable declarations and assignments throughout program execution.
- **Interactive CLI:** Includes a live REPL for real-time coding and a file runner for `.lexor` scripts.

---

## 📂 Project Structure

```text
Lexor/
├── src/
│   ├── main.rs           # Application entry point (REPL & File Runner)
│   ├── lexer.rs          # Tokenization of raw LEXOR source code
│   ├── token.rs          # Token types and Token struct definitions
│   ├── parser.rs         # Recursive-descent parser; builds the AST
│   ├── expr.rs           # Expression node definitions (Expr enum)
│   ├── stmt.rs           # Statement node definitions (Stmt enum)
│   ├── interpreter.rs    # The tree-walking execution engine
│   ├── environment.rs    # Variable scope and storage
│   └── error.rs          # Runtime and syntax error types
├── Cargo.toml            # Rust project manifest and dependencies
└── Cargo.lock            # Dependency lock file
```

---

## 🛠️ How to Build & Run

### Prerequisites

Ensure you have **Rust** installed. If not, install it via [rustup](https://rustup.rs/):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build the Interpreter

Clone the repository and build using Cargo:
```bash
cargo build --release
```

The compiled binary will be at `target/release/Lexor` (Linux/macOS) or `target\release\Lexor.exe` (Windows).

### Running the Interpreter

**Run a `.lexor` script file:**

On Windows:
```cmd
target\release\Lexor.exe my_script.lexor
```

On Mac/Linux:
```bash
./target/release/Lexor my_script.lexor
```

**Launch the interactive REPL:**

Simply run the interpreter with no arguments:
```bash
./target/release/Lexor
```

In the REPL, type your LEXOR program line by line, then type `RUN` on a new line to execute it.

---

## 💻 Sample Programs

### 1. Hello World & Basic I/O

Create a file named `hello.lexor`:

```
SCRIPT AREA
START SCRIPT
  DECLARE INT age = 0
  DECLARE CHAR grade = 'A'

  PRINT: "Hello, LEXOR!" & $
  PRINT: "Enter your age:"
  SCAN: age
  PRINT: "You are " & age & " years old." & $
  PRINT: "Your grade is: " & grade
END SCRIPT
```

### 2. Arithmetic Operations

```
SCRIPT AREA
START SCRIPT
  DECLARE INT xyz, abc = 100
  xyz = ((abc * 5) / 10 + 10) * -1
  PRINT: [[] & xyz & []]
END SCRIPT
```

Output:
```
[-60]
```

### 3. Logical Operations

```
SCRIPT AREA
START SCRIPT
  DECLARE INT a = 100, b = 200, c = 300
  DECLARE BOOL d = "FALSE"
  d = (a < b AND c <> 200)
  PRINT: d
END SCRIPT
```

Output:
```
TRUE
```

### 4. Countdown (REPEAT WHEN Loop)

Create a file named `countdown.lexor`:

```
%% A simple countdown script
SCRIPT AREA
START SCRIPT
  DECLARE INT count, limit = 0
  DECLARE FLOAT multi = 1.5

  PRINT: "Enter a starting number:"
  SCAN: count

  REPEAT WHEN (count > limit)
  START REPEAT
    PRINT: "T-Minus " & count & "..." & $
    count = count - 1
  END REPEAT

  PRINT: "BLASTOFF! Final calc: " & (limit * multi)
END SCRIPT
```

### 5. IF / ELSE Branching

```
SCRIPT AREA
START SCRIPT
  DECLARE INT score = 0
  PRINT: "Enter your score:"
  SCAN: score

  IF (score >= 75)
  START IF
    PRINT: "Passed!" & $
  END IF
  ELSE
  START IF
    PRINT: "Failed." & $
  END IF
END SCRIPT
```

---

## 📖 Language Reference

### Program Structure

| Rule | Description |
|---|---|
| `SCRIPT AREA` | Required header; marks the start of a LEXOR program |
| `START SCRIPT` / `END SCRIPT` | Wraps all executable code |
| `DECLARE` | Variable declarations; must appear right after `START SCRIPT` |
| `%%` | Single-line comment; can appear anywhere |

### Data Types

| Type | Description |
|---|---|
| `INT` | Whole number, 4 bytes (e.g., `42`, `-7`) |
| `FLOAT` | Decimal number, 4 bytes (e.g., `3.14`) |
| `CHAR` | Single character in single quotes (e.g., `'A'`) |
| `BOOL` | Boolean in double quotes — `"TRUE"` or `"FALSE"` |

### Operators

| Category | Operators |
|---|---|
| Arithmetic | `+`, `-`, `*`, `/`, `%`, `( )` |
| Comparison | `>`, `<`, `>=`, `<=`, `==`, `<>` |
| Logical | `AND`, `OR`, `NOT` |
| Unary | `+` (positive), `-` (negative) |
| String / Output | `&` (concatenator), `$` (newline), `[#]` (escape code) |

### I/O Statements

```
%% Output
PRINT: "Hello " & name & $

%% Input (single or multiple variables, comma-separated)
SCAN: x, y, z
```

### Control Flow

```
%% If / Else If / Else
IF (<condition>)
START IF
  ...
END IF
ELSE IF (<condition>)
START IF
  ...
END IF
ELSE
START IF
  ...
END IF

%% For Loop
FOR (<init>, <condition>, <update>)
START FOR
  ...
END FOR

%% While Loop (REPEAT WHEN)
REPEAT WHEN (<condition>)
START REPEAT
  ...
END REPEAT
```

---

## 🏗️ How the Interpreter Works

The interpreter follows a classic three-phase pipeline:

1. **Lexer (`lexer.rs`)** — Scans the raw source string character by character and emits a flat list of `Token` objects. Keywords, literals, operators, and special symbols like `$` and `&` are all recognized here.

2. **Parser (`parser.rs`)** — Consumes the token stream using recursive descent and constructs an **Abstract Syntax Tree (AST)** made up of `Stmt` (statement) and `Expr` (expression) nodes. Syntax errors are reported at this stage.

3. **Interpreter (`interpreter.rs`)** — Walks the AST recursively. Each `Stmt` node is executed and each `Expr` node is evaluated. The `Environment` (`environment.rs`) stores all declared variables and their current values throughout program execution.

---

## 📦 Dependencies

| Crate | Version | Purpose |
|---|---|---|
| `regex` | `1.12.3` | Pattern matching used in the lexer |

---


