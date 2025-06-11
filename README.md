# My First Project on Rust

This is my first project using the Rust programming language! I built a comprehensive file explorer application that runs on your computer. The app lets you browse through folders and files on your system using a clean, easy-to-use graphical interface. You can click on folders to open them, navigate back to parent directories with the "Up" button, and see all your files and folders organized neatly with folder and file icons. You can also click on any file to open it with your computer's default application. The app includes a built-in terminal at the bottom and a settings panel to customize your experience. It's like having a powerful file manager and terminal combined, built from scratch using Rust and a GUI library called eframe.

## What it does
- Browse files and folders on your computer
- Navigate through directories by clicking on folders
- Open files by clicking on them (uses your system's default application)
- Settings panel with a toggle to show/hide hidden files (files starting with '.')
- Built-in terminal at the bottom for running commands
- Smart cd integration - when you use `cd` in the terminal, the file explorer updates to show that directory
- Shows folders first, then files (alphabetically sorted)
- Go back to parent directories with the Up button
- Displays error messages if there are permission issues
- Shows status messages when files are successfully opened

## Terminal Features
- Execute any system command from within the app
- Built-in commands: `cd`, `pwd`, `ls`, `ls -la`, `clear`
- Command history with up/down arrow keys
- When you `cd` to a directory, the file explorer automatically updates
- All commands execute in the current directory shown in the file explorer

## Settings
- Show/Hide hidden files: Toggle visibility of files and folders starting with '.'
- Access via the ⚙ Settings button in the top-right corner

## How to run
Make sure you have Rust installed on your computer, then:

```bash
cargo run
```

## Dependencies
- eframe 0.27.2 (for the graphical interface)
- open 5.0 (for opening files with default applications)

## Screenshot
The app opens a window with three main areas:
- Top: Navigation bar with Up button, current path, and Settings button
- Middle: File explorer showing clickable folders (📁) and files (📄)
- Bottom: Resizable terminal panel for running commands

Try typing `ls`, `pwd`, or `cd ..` in the terminal! 