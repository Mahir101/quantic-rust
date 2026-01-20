# Contributing to Quantic-Rust

Thank you for your interest in Quantic-Rust! We welcome contributions from the community to help make this high-performance quantum computing library even better.

## How to Contribute

### Reporting Bugs
- Search existing issues to see if the bug has already been reported.
- If not, open a new issue using the **Bug Report** template.
- Provide a clear description, steps to reproduce, and any relevant logs or system information.

### Suggesting Enhancements
- Check the project's roadmap and existing issues.
- Open a new issue using the **Feature Request** template.
- Explain the motivation and provide examples of how the feature would be used.

### Code Contributions
1. Fork the repository.
2. Create a new branch: `git checkout -b feature/your-feature-name`.
3. Make your changes.
4. Ensure tests pass (see [Development Setup](#development-setup)).
5. Submit a pull request.

## Development Setup

Quantic-Rust is a hybrid project with Rust core and Python bindings using `maturin`.

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Python](https://www.python.org/downloads/) (>= 3.9)
- [maturin](https://github.com/PyO3/maturin)

### Building from Source
```bash
# Install development dependencies
pip install maturin pytest numpy networkx

# Build and install in development mode
maturin develop
```

### Running Tests
```bash
# Run Rust tests
cargo test

# Run Python tests
pytest
```

## Pull Request Guidelines
- Follow the existing code style.
- Include unit tests for new features.
- Update documentation in `README.md` or Docstrings if necessary.
- Ensure the PR description clearly explains the changes.

## Code of Conduct
Please note that this project is released with a [Contributor Code of Conduct](CODE_OF_CONDUCT.md). By participating in this project you agree to abide by its terms.
