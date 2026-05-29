@echo off
rem 🛡️ Git Hooks Setup Script for Windows

echo 🚀 Setting up Git Hooks for Windows...

rem Configure git to use our custom hooks directory
git config core.hooksPath .githooks

echo ✅ Git hooks configured successfully! Git will now run automated checks on 'git commit'.
pause
