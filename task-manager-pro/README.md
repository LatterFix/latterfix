# Soroban Project

## Project Structure

This repository uses the recommended structure for a Soroban project:

```text
.
├── contracts
│   └── hello_world
│       ├── src
│       │   ├── lib.rs
│       │   └── test.rs
│       └── Cargo.toml
├── Cargo.toml
└── README.md
```

- New Soroban contracts can be put in `contracts`, each in their own directory. There is already a `hello_world` contract in there to get you started.
- If you initialized this project with any other example contracts via `--with-example`, those contracts will be in the `contracts` directory as well.
- Contracts should have their own `Cargo.toml` files that rely on the top-level `Cargo.toml` workspace for their dependencies.
- Frontend libraries can be added to the top-level directory as well. If you initialized this project with a frontend template via `--frontend-template` you will have those files already included.

// 1: feat: add task detail page with contributor flow

// 2: feat: implement project manager dashboard

// 3: feat: add dark mode with persistent preference

// 4: fix: resolve Freighter signing timeout

// 5: feat: add task category tags and filters

// 6: feat: implement real-time bid counter

// 7: feat: add contributor portfolio page

// 8: fix: correct timezone display for deadlines

// 9: feat: add mobile responsive task board
