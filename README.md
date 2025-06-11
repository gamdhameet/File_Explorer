# My First Project on Rust

This is my first project using the Rust programming language! I built a simple file explorer application that runs on your computer. The app lets you browse through folders and files on your system using a clean, easy-to-use graphical interface. You can click on folders to open them, navigate back to parent directories with the "Up" button, and see all your files and folders organized neatly with folder and file icons. It's like having a basic file manager built from scratch using Rust and a GUI library called eframe.

## What it does
- Browse files and folders on your computer
- Navigate through directories by clicking
- Shows folders first, then files (alphabetically sorted)
- Go back to parent directories with the Up button
- Displays error messages if there are permission issues

## How to run
Make sure you have Rust installed on your computer, then:

```bash
cargo run
```

## Dependencies
- eframe 0.27.2 (for the graphical interface)

## Screenshot
The app opens a window showing your current directory with clickable folders (��) and files (📄). 