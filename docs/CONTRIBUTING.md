## How to Contribute

We welcome contributions to `rgrim`! Please ensure your work aligns with our formatting and workflow structures:

### 1. Branch Naming & Commit Style

To maintain a clean project history, we strictly enforce the **Conventional Commits** specification for both branch naming and commit messages.

* **Branches:** Use semantic prefixes followed by the ticket number or a short description (e.g., `feat/export-and-clipboard`, `fix/monitor-size`, `docs/update-readme`).
* **Commits:** Structure messages cleanly using a structural indicator (e.g., `feat: add notify-rust tracking channel`, `fix(ui): resolve coordinate displacement bounds under wayland`).

### 2. Pull Request Format Requirements

When opening a Pull Request, the description must strictly include the following sections to assist reviewers:

* **Clear Title & Context:** Use structured ticket brackets within the title layout (e.g., `[UI-145] Fix empty state in cart`).
* **Essence of Changes:** Provide a brief explanation describing **what** was altered, **why** it was necessary, and **how** it was technically implemented.
* **Traceability Links:** Include working links pointing to tracking tickets, bug reports, mockups, or related community discussions.
* **Visual Evidence:** Attach relevant screenshots or recorded captures if your adjustments introduce changes to the UI layer.
* **Self-Check Checklist:** Complete a validation checklist before requesting a peer review:
* [ ] cargo check
* [ ] cargo fmt
* [ ] cargo build
* [ ] Tested successfully in staging/local container environment.
* [ ] Verified regression criteria and added regression tests.
* [ ] Updated companion documentation assets accordingly.
