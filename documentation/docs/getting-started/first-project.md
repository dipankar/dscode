# Your First Project

In this tutorial you will build a small web page from scratch using DSCode.
Along the way you will use the editor, the integrated terminal, extensions,
and Git integration -- the core workflows that make DSCode productive for
real-world development.

**Time required:** ~15 minutes

---

## Step 1: Create a Project Directory

Open your system terminal (or the DSCode integrated terminal if DSCode is
already running) and create a new directory:

```bash
mkdir ~/my-first-project
cd ~/my-first-project
```

---

## Step 2: Open the Project in DSCode

Open the folder in DSCode using one of these methods:

=== "Terminal"

    ```bash
    dscode ~/my-first-project
    ```

=== "Menu"

    In DSCode, go to **File > Open Folder...** and select `my-first-project`.

=== "Keyboard"

    Press ++ctrl+o++ (++cmd+o++ on macOS) and navigate to the folder.

The Explorer sidebar should now show an empty project.

---

## Step 3: Create Your Files

You will create three files: an HTML page, a CSS stylesheet, and a JavaScript
file.

### index.html

1. In the Explorer sidebar, click the **New File** icon (or press
   ++ctrl+n++ / ++cmd+n++).
2. Name the file `index.html`.
3. Type `!` and press ++tab++ to expand the **Emmet** abbreviation into a
   full HTML5 boilerplate.

!!! tip "Emmet in action"
    DSCode includes Emmet support out of the box. Try typing
    `ul>li*5>a{Item $}` and pressing ++tab++ -- it expands into a full
    unordered list with five linked items.

Replace the contents of `<body>` with:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>My First DSCode Project</title>
    <link rel="stylesheet" href="style.css" />
</head>
<body>
    <div class="container">
        <h1>Hello from DSCode!</h1>
        <p id="message">Click the button to get started.</p>
        <button id="action-btn">Click Me</button>
    </div>
    <script src="app.js"></script>
</body>
</html>
```

Save the file with ++ctrl+s++ (++cmd+s++ on macOS).

### style.css

Create a new file called `style.css` and add:

```css
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: "Inter", system-ui, sans-serif;
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 100vh;
    background: linear-gradient(135deg, #667eea, #764ba2);
    color: #fff;
}

.container {
    text-align: center;
    padding: 2rem;
}

h1 {
    font-size: 2.5rem;
    margin-bottom: 1rem;
}

p {
    font-size: 1.2rem;
    margin-bottom: 2rem;
    opacity: 0.9;
}

button {
    padding: 0.75rem 2rem;
    font-size: 1rem;
    border: 2px solid #fff;
    border-radius: 8px;
    background: transparent;
    color: #fff;
    cursor: pointer;
    transition: all 0.2s ease;
}

button:hover {
    background: #fff;
    color: #764ba2;
}
```

### app.js

Create a new file called `app.js` and add:

```javascript
const button = document.getElementById("action-btn");
const message = document.getElementById("message");

let clickCount = 0;

button.addEventListener("click", () => {
    clickCount++;
    message.textContent = `You clicked ${clickCount} time${clickCount === 1 ? "" : "s"}!`;
});
```

---

## Step 4: Explore Editor Features

Now that you have three files open, try out some of the editing features
that make DSCode powerful.

### IntelliSense

Open `app.js` and start typing `document.query`. DSCode will show
autocomplete suggestions powered by the Monaco editor's built-in
JavaScript/TypeScript language service.

- Press ++enter++ or ++tab++ to accept a suggestion.
- Press ++escape++ to dismiss the autocomplete menu.
- Press ++ctrl+space++ (++cmd+space++ on macOS) to manually trigger
  suggestions at any time.

### Multi-Cursor Editing

1. Open `style.css`.
2. Select the word `margin` on any line.
3. Press ++ctrl+d++ (++cmd+d++ on macOS) repeatedly to select the next
   occurrence.
4. Type a replacement -- all cursors update simultaneously.

Alternatively, hold ++alt++ (++option++ on macOS) and click at multiple
positions to place cursors manually.

### Emmet Abbreviations

Open `index.html`, place your cursor inside `<div class="container">`, and
try typing:

```
p.description{This is a new paragraph}
```

Press ++tab++ to expand it into:

```html
<p class="description">This is a new paragraph</p>
```

---

## Step 5: Run a Local Server

Open the integrated terminal with ++ctrl+grave++ (++cmd+grave++ on macOS).

If you have Python installed, start a quick HTTP server:

=== "Python 3"

    ```bash
    python3 -m http.server 8080
    ```

=== "Node.js (npx)"

    ```bash
    npx serve -l 8080
    ```

=== "PHP"

    ```bash
    php -S localhost:8080
    ```

Open `http://localhost:8080` in your browser to see your page in action.
Click the button to verify the JavaScript is working.

!!! tip "Live reload"
    For a better development experience, install a live-reload server:

    ```bash
    npm install -g live-server
    live-server --port=8080
    ```

    Changes you save in DSCode will automatically refresh the browser.

---

## Step 6: Install an Extension

Let us install an extension to improve the development workflow.

1. Open the Extensions sidebar with ++ctrl+shift+x++ (++cmd+shift+x++ on
   macOS).
2. Search for **Live Preview**.
3. Click **Install**.

Once installed, you can right-click `index.html` in the Explorer and select
**Open with Live Preview** to see a real-time preview inside DSCode.

!!! tip "Recommended extensions for web development"
    - **Prettier** -- Automatic code formatting
    - **ESLint** -- JavaScript linting
    - **Auto Rename Tag** -- Rename matching HTML tags
    - **CSS Peek** -- Jump to CSS definitions from HTML

    See [Recommended Extensions](../extensions/recommended-extensions.md) for
    a curated list.

---

## Step 7: Initialize Git and Make Your First Commit

DSCode has built-in Git integration. Let us set up version control for this
project.

### Initialize the Repository

Open the integrated terminal (++ctrl+grave++ / ++cmd+grave++) and run:

```bash
git init
```

You will notice the Explorer sidebar now shows file status indicators --
untracked files appear with a **U** badge.

### Stage Your Files

You can stage files in two ways:

=== "Source Control Panel"

    1. Click the **Source Control** icon in the activity bar (or press
       ++ctrl+shift+g++ / ++cmd+shift+g++).
    2. Hover over each file under **Changes** and click the **+** icon to
       stage it.
    3. Or click the **+** next to the **Changes** header to stage all files
       at once.

=== "Terminal"

    ```bash
    git add -A
    ```

### Commit

1. In the Source Control panel, type a commit message in the text field at
   the top:

    ```
    Initial commit: add HTML, CSS, and JS files
    ```

2. Press ++ctrl+enter++ (++cmd+enter++ on macOS) or click the **Commit**
   button.

!!! info "Git configuration"
    If this is your first time using Git, you may need to configure your
    identity:

    ```bash
    git config --global user.name "Your Name"
    git config --global user.email "you@example.com"
    ```

### View History

Open the Command Palette (++ctrl+shift+p++ / ++cmd+shift+p++) and type
`Git: View History` to see your commit log.

---

## What You Learned

In this tutorial you covered the fundamental DSCode workflows:

- [x] Creating and opening a project
- [x] Creating and editing files with syntax highlighting
- [x] Using IntelliSense for autocomplete
- [x] Multi-cursor editing for efficient changes
- [x] Emmet abbreviations for rapid HTML authoring
- [x] Running commands in the integrated terminal
- [x] Installing extensions to enhance functionality
- [x] Initializing a Git repository, staging, and committing

---

## Next Steps

You now have a solid foundation. Here are some areas to explore next:

- **[Editor Features](../user-guide/editor-features.md)** -- Deep dive into
  code navigation, refactoring, snippets, and more.
- **[Integrated Terminal](../user-guide/integrated-terminal.md)** -- Split
  terminals, shell profiles, and task automation.
- **[Git Integration](../user-guide/git-integration.md)** -- Branching,
  merging, diff viewing, and remote workflows.
- **[Search](../user-guide/search.md)** -- Project-wide search and replace
  with regex support.
- **[Settings and Configuration](../user-guide/settings-and-configuration.md)**
  -- Customize DSCode to match your preferences.
- **[Keyboard Shortcuts](../user-guide/keyboard-shortcuts.md)** -- Master the
  shortcuts that will make you faster.
- **[Installing Extensions](../extensions/installing-extensions.md)** --
  Browse the marketplace and manage your extension library.
