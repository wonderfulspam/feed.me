# Project Review

This project review was done by gemini-2.5-pro through gemini-cli v0.2.2.

## Prompt

I want you to take a critical view of the codebase and especially focus on how I
can clean it up so it becomes more extensible and easier to maintain. I want you
to check all the markdown documents and put them into a state where they reflect
what's left to do; I don't care about past accomplishments. Check the github
workflows - are they easy to maintain or can we move more things to local
workflows? I particularly struggle with releasing new versions - I often forget
to bump @spacefeeder/Cargo.toml and afterwards also @spacefeeder/Cargo.lock. The
performance of the workflows is currently excellent so I'd like to retain that.
For the "frontend", are the templates generic such that I can easily redesign
the website in the future? Will it be easy for me to add interactivity to the
currently static site if I decide to host it? For the backend, I got rid of Zola
and moved website generation into spacefeeder/, but is the software organised in
a way where it is easy to extend and expand? For the CLI, do the commands still
do the right thing after we implemented parts of @COMPLEX_CATEGORISATION.md? We
want to keep editing @spacefeeder.toml but all feeds we like and love should
"graduate" and be promoted to @data/feeds.toml, and the other @data/** files
should similarly keep getting organically larger and more refined. Maybe we need
some slash commands for agents that instruct them to sample the output from
fetching feeds and seeing if they are correctly categorized? Or other ways to
find mismatches more statically in a lightweight, rust-first way? I want you to
start by reading @CLAUDE.md and @README.md. Then create a file called
PROJECT_REVIEW.md where you write this exact prompt in the intro session, and
then capture your understanding of all the tasks I've just given you. Feel free
to read a few files to get a better idea of the project before you write the
task list.

## Task Breakdown

This is my understanding of the tasks you have given me. I will use this as a
checklist to guide the project review.

### 1. General Code Health and Maintainability
- **Goal:** Critically assess the entire codebase for cleanliness, extensibility, and ease of maintenance.
- **Actions:**
    - Identify areas for refactoring.
    - Suggest improvements to the overall architecture.
    - Look for code smells, anti-patterns, or overly complex sections.

### 2. Documentation Cleanup
- **Goal:** Update all Markdown documents to be forward-looking.
- **Actions:**
    - Read all `.md` files.
    - Remove sections detailing past accomplishments or completed tasks.
    - Re-center the documentation on what is planned or "left to do".

### 3. Workflow and Release Process
- **Goal:** Simplify the release process and improve workflow maintainability.
- **Actions:**
    - Analyze `.github/workflows/` files.
    - Propose moving CI/CD logic to local scripts (e.g., `justfile`) where appropriate, while retaining performance.
    - Investigate solutions to automate version bumping for `spacefeeder/Cargo.toml` and `spacefeeder/Cargo.lock` during releases.

### 4. Frontend Extensibility
- **Goal:** Evaluate the flexibility of the frontend templates for future changes.
- **Actions:**
    - Review the Tera templates in `templates/`.
    - Assess how easily the site can be visually redesigned without major structural changes.
    - Determine the feasibility of adding client-side interactivity (e.g., JavaScript) to the statically generated pages.

### 5. Backend Architecture
- **Goal:** Review the organization of the `spacefeeder` Rust crate for extensibility.
- **Actions:**
    - Analyze the module structure within `spacefeeder/src/`.
    - Evaluate how well the code is organized for adding new features or modifying existing ones, especially after the removal of Zola.

### 6. CLI Command Verification
- **Goal:** Ensure CLI commands are functioning correctly and align with the new feed management strategy.
- **Actions:**
    - Review the implementation of CLI commands in `spacefeeder/src/commands/`.
    - Verify their behavior against the logic described in `COMPLEX_CATEGORISATION.md`.
    - Check the process for "graduating" feeds from `spacefeeder.toml` to `data/feeds.toml`.

### 7. Categorization and Data Refinement

- **Goal:** Improve the feed categorization system and the process for refining its data.
- **Actions:**
    - Analyze the current categorization logic and its data sources (`data/*.toml`).
    - Propose methods to identify categorization mismatches, potentially through:
        - New CLI commands or agent "slash commands" for sampling and validation.
        - Lightweight, Rust-first static analysis tools.
