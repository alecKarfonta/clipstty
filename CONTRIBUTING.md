# Contributing to STT Clippy

Thank you for your interest in contributing to STT Clippy! This document provides guidelines and information for contributors.

## 🤝 How to Contribute

There are many ways to contribute to STT Clippy:

- **🐛 Report Bugs**: Help us identify and fix issues
- **💡 Suggest Features**: Propose new ideas and improvements
- **🔧 Fix Issues**: Pick up existing issues and submit fixes
- **📝 Improve Documentation**: Help make our docs clearer and more comprehensive
- **🧪 Add Tests**: Improve test coverage and reliability
- **🌍 Localization**: Help translate the application to other languages
- **🔍 Code Review**: Review pull requests and provide feedback

## 🚀 Getting Started

### Prerequisites

- **Git**: Version control system
- **Rust**: 1.70+ (install from [rustup.rs](https://rustup.rs/))
- **Node.js**: 18+ (for UI development)
- **Platform-specific tools** (see [README.md](README.md) for details)

### Setting Up Your Development Environment

1. **Fork the repository**
   ```bash
   # Go to https://github.com/your-org/stt-clippy and click "Fork"
   ```

2. **Clone your fork**
   ```bash
   git clone https://github.com/YOUR_USERNAME/stt-clippy.git
   cd stt-clippy
   ```

3. **Add the upstream remote**
   ```bash
   git remote add upstream https://github.com/your-org/stt-clippy.git
   ```

4. **Install dependencies**
   ```bash
   cargo build
   ```

5. **Run tests to ensure everything works**
   ```bash
   cargo test
   ```

## 🔧 Development Workflow

### 1. Choose an Issue

- Check the [issue tracker](https://github.com/your-org/stt-clippy/issues) for open issues
- Look for issues labeled `good first issue` if you're new to the project
- Comment on issues you'd like to work on to avoid duplication

### 2. Create a Feature Branch

```bash
# Update your main branch
git checkout main
git pull upstream main

# Create a new branch for your work
git checkout -b feature/your-feature-name
```

**Branch Naming Convention:**
- `feature/feature-name` - for new features
- `fix/issue-description` - for bug fixes
- `docs/documentation-update` - for documentation changes
- `test/test-improvement` - for test-related changes

### 3. Make Your Changes

- Write clear, readable code
- Follow the existing code style and conventions
- Add tests for new functionality
- Update documentation as needed
- Keep commits focused and atomic

### 4. Test Your Changes

```bash
# Run the test suite
cargo test

# Run tests with coverage
cargo tarpaulin

# Run integration tests
cargo test --test integration

# Run performance benchmarks
cargo bench

# Check code formatting
cargo fmt --check

# Run clippy (Rust linter)
cargo clippy
```

### 5. Commit Your Changes

```bash
# Stage your changes
git add .

# Commit with a descriptive message
git commit -m "feat: add new STT backend support

- Add support for Deepgram STT API
- Implement streaming transcription
- Add configuration options for API keys
- Include comprehensive tests

Closes #123"
```

**Commit Message Format:**
We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

- `feat:` - new features
- `fix:` - bug fixes
- `docs:` - documentation changes
- `style:` - formatting, missing semicolons, etc.
- `refactor:` - code refactoring
- `test:` - adding or updating tests
- `chore:` - maintenance tasks

### 6. Push and Create a Pull Request

```bash
# Push your branch to your fork
git push origin feature/your-feature-name
```

Then go to your fork on GitHub and create a Pull Request.

## 📋 Pull Request Guidelines

### Before Submitting

- [ ] **Tests pass**: All tests should pass locally
- [ ] **Code coverage**: New code should have test coverage
- [ ] **Documentation**: Update relevant documentation
- [ ] **Code style**: Follow the project's coding standards
- [ ] **Single responsibility**: Each PR should address one issue/feature

### Pull Request Template

When creating a PR, use this template:

```markdown
## Description
Brief description of what this PR accomplishes.

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

## Testing
- [ ] I have tested this change locally
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] All tests pass

## Checklist
- [ ] My code follows the style guidelines of this project
- [ ] I have performed a self-review of my own code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] I have made corresponding changes to the documentation
- [ ] My changes generate no new warnings
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally with my changes

## Related Issues
Closes #123
```

## 🎨 Code Style and Standards

### Rust Code Style

- Follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/style/naming/README.html)
- Use `cargo fmt` to format your code
- Run `cargo clippy` to check for common issues
- Prefer `clippy::pedantic` warnings

### General Guidelines

- **Readability**: Write code that's easy to understand
- **Documentation**: Document public APIs and complex logic
- **Error Handling**: Use proper error types and handle errors gracefully
- **Performance**: Consider performance implications of your changes
- **Security**: Follow security best practices

### Naming Conventions

- **Functions**: `snake_case`
- **Variables**: `snake_case`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Types**: `PascalCase`
- **Files**: `snake_case.rs`

## 🧪 Testing Guidelines

### Test Coverage

- Aim for >80% code coverage
- Write tests for all new functionality
- Include edge cases and error conditions
- Test both success and failure paths

### Test Types

- **Unit Tests**: Test individual functions and methods
- **Integration Tests**: Test component interactions
- **Performance Tests**: Benchmark critical code paths
- **End-to-End Tests**: Test complete user workflows

### Test Naming

```rust
#[test]
fn test_function_name_with_expected_behavior() {
    // Test implementation
}

#[test]
fn test_function_name_handles_error_case() {
    // Error handling test
}
```

## 📚 Documentation Standards

### Code Documentation

- Document all public APIs with doc comments
- Include examples in doc comments
- Use clear, concise language
- Follow Rust documentation conventions

```rust
/// Transcribes audio data to text using the specified STT backend.
///
/// # Arguments
///
/// * `audio_data` - Raw audio data in 16-bit PCM format
/// * `backend` - The STT backend to use for transcription
///
/// # Returns
///
/// Returns a `Result` containing the transcribed text or an error.
///
/// # Examples
///
/// ```rust
/// use stt_clippy::stt::STTService;
///
/// let service = STTService::new();
/// let result = service.transcribe(&audio_data, "local");
/// ```
pub fn transcribe(&self, audio_data: &[u8], backend: &str) -> Result<String, STTError> {
    // Implementation
}
```

### README and Documentation

- Keep documentation up to date with code changes
- Use clear, simple language
- Include code examples
- Add screenshots for UI changes

## 🐛 Bug Reports

### Before Reporting

- Check if the issue has already been reported
- Try to reproduce the issue with the latest version
- Check the documentation and existing issues

### Bug Report Template

```markdown
## Bug Description
Clear and concise description of the bug.

## Steps to Reproduce
1. Go to '...'
2. Click on '...'
3. Scroll down to '...'
4. See error

## Expected Behavior
What you expected to happen.

## Actual Behavior
What actually happened.

## Environment
- OS: [e.g. Ubuntu 22.04]
- Version: [e.g. 1.0.0]
- Hardware: [e.g. CPU, RAM, GPU]

## Additional Context
Any other context about the problem.
```

## 💡 Feature Requests

### Feature Request Guidelines

- **Clear Description**: Explain what you want and why
- **Use Cases**: Provide specific examples of how it would be used
- **Alternatives**: Consider if there are existing ways to achieve this
- **Implementation**: Suggest how it might be implemented (if you have ideas)

### Feature Request Template

```markdown
## Feature Description
Clear and concise description of the feature.

## Problem Statement
What problem does this feature solve?

## Proposed Solution
How would you like this feature to work?

## Use Cases
Provide specific examples of when this feature would be useful.

## Alternatives Considered
What alternatives have you considered?

## Additional Context
Any other context or screenshots about the feature request.
```

## 🔍 Code Review Process

### Review Guidelines

- **Be Constructive**: Provide helpful, actionable feedback
- **Focus on Code**: Review the code, not the person
- **Ask Questions**: If something isn't clear, ask for clarification
- **Suggest Improvements**: Offer specific suggestions for improvement

### Review Checklist

- [ ] **Functionality**: Does the code do what it's supposed to do?
- [ ] **Performance**: Are there performance implications?
- [ ] **Security**: Are there security concerns?
- [ ] **Testing**: Is the code adequately tested?
- [ ] **Documentation**: Is the code properly documented?
- [ ] **Style**: Does the code follow project conventions?

## 🚀 Release Process

### Release Cycle

- **Patch Releases**: Bug fixes and minor improvements
- **Minor Releases**: New features and improvements
- **Major Releases**: Breaking changes and major features

### Release Checklist

- [ ] All tests pass
- [ ] Documentation is updated
- [ ] Changelog is updated
- [ ] Version numbers are updated
- [ ] Release notes are written
- [ ] Binaries are built and tested
- [ ] Release is tagged and published

## 📞 Getting Help

### Communication Channels

- **GitHub Issues**: For bug reports and feature requests
- **GitHub Discussions**: For questions and general discussion
- **Discord**: For real-time chat (if available)
- **Email**: For sensitive issues or private discussions

### Asking for Help

When asking for help:

- **Be Specific**: Describe your problem clearly
- **Provide Context**: Include relevant code and error messages
- **Show Effort**: Demonstrate what you've already tried
- **Be Patient**: Contributors are volunteers with limited time

## 🙏 Recognition

### Contributors

- All contributors are listed in the [CONTRIBUTORS.md](CONTRIBUTORS.md) file
- Significant contributions are recognized in release notes
- Contributors are mentioned in project documentation

### Contribution Levels

- **Contributor**: Any contribution to the project
- **Maintainer**: Regular contributor with commit access
- **Core Team**: Long-term contributor with project leadership

## 📄 License

By contributing to STT Clippy, you agree that your contributions will be licensed under the same license as the project (MIT License).

---

Thank you for contributing to STT Clippy! Your contributions help make this project better for everyone. 🎉

If you have any questions about contributing, feel free to open an issue or start a discussion.
