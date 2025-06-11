# My First Project on Rust

This is my first project using the Rust programming language! I built a comprehensive, professional-grade file explorer application that runs on your computer. The app provides a complete file management experience with a modern graphical interface, built-in terminal, and powerful features. You can browse folders, manage files with full copy/cut/paste operations, use keyboard shortcuts, customize the interface with different themes and view modes, save bookmarks, and much more. It's like having a feature-rich file manager with an integrated terminal, built entirely from scratch using Rust and the eframe GUI library.

## 🚀 Core Features

### **File Operations**
- **Right-click context menu** with Copy, Cut, Paste, Delete, Rename, Properties
- **Keyboard shortcuts**: Ctrl+C (copy), Ctrl+X (cut), Ctrl+V (paste), Del (delete), F2 (rename)
- **Create new files and folders** with dedicated buttons
- **File properties dialog** showing size, permissions, and modification date
- **Multi-file selection** with Ctrl+click
- **Open files** by clicking (uses system's default application)

### **Enhanced Navigation**
- **Breadcrumb navigation** - clickable path components in the top bar
- **Back/Forward buttons** with full navigation history
- **Bookmarks/Favorites** - save and quickly access frequently used directories
- **Up button** to navigate to parent directories
- **Smart cd integration** - terminal commands update the file explorer

### **Visual Improvements**
- **Different file type icons**: 📄 .txt, 🎵 .mp3, 🖼️ .jpg, 💻 .py/.rs/.js, 📕 .pdf, and more
- **File size and date columns** in list view with proper formatting (KB/MB/GB)
- **Dark/Light theme toggle** for comfortable viewing
- **Grid view vs List view** options for different browsing preferences
- **Professional layout** with organized panels and toolbars

## 📋 Interface Layout

### **Top Panel**
- **Navigation**: Back/Forward buttons, Up button, clickable breadcrumb path
- **Actions**: New File, New Folder buttons
- **View Options**: List/Grid view toggles, Light/Dark theme selection
- **Quick Access**: Settings and Bookmarks panels

### **Main Content Area**
- **List View**: Detailed view with icons, names, sizes, and modification dates
- **Grid View**: Large icon view for visual browsing
- **Multi-selection**: Ctrl+click to select multiple files
- **Context menus**: Right-click for file operations

### **Bottom Terminal**
- **Built-in terminal** for command execution
- **Command history** with up/down arrow navigation
- **Smart directory sync** - cd commands update the file explorer
- **Resizable panel** - drag to adjust height

## ⌨️ Keyboard Shortcuts
- **Ctrl+C**: Copy selected files
- **Ctrl+X**: Cut selected files  
- **Ctrl+V**: Paste files
- **Delete**: Delete selected files
- **F2**: Rename selected file
- **Enter**: Execute terminal command

## 💾 Advanced Features

### **Bookmarks System**
- Add current directory as bookmark with custom name
- Quick navigation to saved locations
- Remove bookmarks with one click
- Bookmarks saved automatically to file

### **Terminal Integration**
- Execute any system command from within the app
- Built-in commands: `cd`, `pwd`, `ls`, `ls -la`, `clear`
- When you `cd` to a directory, the file explorer automatically updates
- All commands execute in the current directory shown in the file explorer
- Command history persists during session

### **File Management**
- Copy and paste files within the application
- Cut and move files to different locations
- Create new files and folders from the interface
- Rename files and folders inline
- Delete files and directories (with confirmation)
- View detailed file properties

### **Customization**
- **Show/Hide hidden files**: Toggle visibility of files starting with '.'
- **Theme switching**: Light and dark modes
- **View modes**: Choose between detailed list or large icon grid
- **Settings panel**: Easy access to preferences

## 🛠️ How to run
Make sure you have Rust installed on your computer, then:

```bash
cargo run
```

## 📦 Dependencies
- **eframe 0.27.2** - GUI framework for the interface
- **open 5.0** - Opening files with default applications
- **arboard 3.3** - Clipboard operations for copy/paste
- **serde 1.0** - Serialization for bookmarks
- **serde_json 1.0** - JSON handling for bookmark storage
- **chrono 0.4** - Date/time formatting for file metadata

## 🎯 Usage Examples

### **File Operations**
1. Select files with click or Ctrl+click for multiple
2. Right-click for context menu or use keyboard shortcuts
3. Create new files/folders with the toolbar buttons
4. Rename files by pressing F2 or right-click → Rename

### **Navigation**
1. Click on breadcrumb path components to jump to parent directories
2. Use Back/Forward buttons to navigate through history
3. Add bookmarks for frequently accessed directories
4. Use terminal `cd` commands to navigate programmatically

### **Terminal**
1. Type commands in the terminal at the bottom
2. Use `cd` to change directories (updates file explorer)
3. Use `ls` to list files in terminal format
4. Press ↑/↓ to browse command history

## 📸 Screenshot
The app opens with a professional three-panel layout:
- **Top**: Navigation breadcrumbs, action buttons, view options, theme toggle
- **Middle**: File listing with customizable List/Grid views and file type icons
- **Bottom**: Integrated terminal with command execution

This is a fully-featured file manager that rivals commercial applications! 🎉 