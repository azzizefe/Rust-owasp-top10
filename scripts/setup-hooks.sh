#!/bin/bash

# 🛡️ Git Hooks Setup Script for Linux / macOS

echo "🚀 Setting up Git Hooks..."

# Make hook executable
chmod +x .githooks/pre-commit

# Configure git to use our custom hooks directory
git config core.hooksPath .githooks

echo "✅ Git hooks configured successfully! Git will now run automated checks on 'git commit'."
