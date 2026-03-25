---
title: Language Providers
description: Implement all 26 language provider types in DSCode -- from completions and hover to semantic tokens and call hierarchies. Code examples for every provider.
---

# Language Providers

Language providers are the backbone of intelligent editing in DSCode. They power IntelliSense, go-to-definition, code actions, formatting, and much more. DSCode supports all **26 provider types** from the VS Code Extension API.

Every provider follows the same pattern:

1. Create a class implementing the provider interface
2. Register it with `vscode.languages.register*Provider(selector, provider)`
3. DSCode calls your provider methods when the user triggers the corresponding feature

---

## Document Selectors

All provider registrations require a **document selector** that specifies which files the provider applies to.

```typescript
// Match a single language
const selector: vscode.DocumentSelector = 'python';

// Match language + scheme
const selector: vscode.DocumentSelector = {
    language: 'typescript',
    scheme: 'file'
};

// Match multiple languages
const selector: vscode.DocumentSelector = [
    { language: 'javascript', scheme: 'file' },
    { language: 'typescript', scheme: 'file' },
    { language: 'javascriptreact', scheme: 'file' },
    { language: 'typescriptreact', scheme: 'file' }
];

// Match by file pattern
const selector: vscode.DocumentSelector = {
    language: 'json',
    pattern: '**/package.json'
};
```

!!! tip "Scheme matters"
    Use `scheme: 'file'` to restrict to files on disk. Omitting it includes untitled files, Git diff views, and other virtual documents.

---

## Providers Reference

### CompletionItemProvider

Provides IntelliSense suggestions as the user types.

```typescript
class MyCompletionProvider implements vscode.CompletionItemProvider {
    provideCompletionItems(
        document: vscode.TextDocument,
        position: vscode.Position,
        token: vscode.CancellationToken,
        context: vscode.CompletionContext
    ): vscode.CompletionItem[] {
        const items: vscode.CompletionItem[] = [];

        const snippet = new vscode.CompletionItem('forEach');
        snippet.kind = vscode.CompletionItemKind.Snippet;
        snippet.insertText = new vscode.SnippetString(
            'forEach((${1:item}) => {\n\t$0\n})'
        );
        snippet.documentation = new vscode.MarkdownString(
            'Iterates over each element in the array.'
        );
        items.push(snippet);

        return items;
    }

    resolveCompletionItem(item: vscode.CompletionItem): vscode.CompletionItem {
        // Add additional details lazily (documentation, detail, additionalTextEdits)
        item.detail = 'Array.prototype.forEach';
        return item;
    }
}

// Register with trigger characters
const disposable = vscode.languages.registerCompletionItemProvider(
    { language: 'javascript', scheme: 'file' },
    new MyCompletionProvider(),
    '.', '(' // trigger characters
);
```

---

### HoverProvider

Shows information when the user hovers over a symbol.

```typescript
class MyHoverProvider implements vscode.HoverProvider {
    provideHover(
        document: vscode.TextDocument,
        position: vscode.Position,
        token: vscode.CancellationToken
    ): vscode.Hover | undefined {
        const range = document.getWordRangeAtPosition(position);
        const word = document.getText(range);

        if (word === 'useState') {
            return new vscode.Hover(
                new vscode.MarkdownString(
                    '```typescript\nfunction useState<S>(initialState: S): [S, Dispatch<SetStateAction<S>>]\n```\n\n' +
                    'Returns a stateful value, and a function to update it.'
                ),
                range
            );
        }
    }
}

vscode.languages.registerHoverProvider('typescriptreact', new MyHoverProvider());
```

---

### DefinitionProvider

Navigates to the definition of a symbol (Go to Definition).

```typescript
class MyDefinitionProvider implements vscode.DefinitionProvider {
    provideDefinition(
        document: vscode.TextDocument,
        position: vscode.Position,
        token: vscode.CancellationToken
    ): vscode.Definition | undefined {
        const word = document.getText(
            document.getWordRangeAtPosition(position)
        );

        // Look up symbol in your index
        const location = this.symbolIndex.get(word);
        if (location) {
            return new vscode.Location(
                vscode.Uri.file(location.file),
                new vscode.Position(location.line, location.column)
            );
        }
    }
}

vscode.languages.registerDefinitionProvider('python', new MyDefinitionProvider());
```

---

### DeclarationProvider

Navigates to the declaration of a symbol (Go to Declaration). Distinct from definition in languages like C/C++ where declarations and definitions can be separate.

```typescript
class MyDeclarationProvider implements vscode.DeclarationProvider {
    provideDeclaration(
        document: vscode.TextDocument,
        position: vscode.Position,
        token: vscode.CancellationToken
    ): vscode.Declaration | undefined {
        // Return the header file declaration
        return new vscode.Location(
            vscode.Uri.file('/project/include/module.h'),
            new vscode.Position(42, 0)
        );
    }
}

vscode.languages.registerDeclarationProvider('c', new MyDeclarationProvider());
```

---

### TypeDefinitionProvider

Navigates to the type definition of a symbol (Go to Type Definition).

```typescript
class MyTypeDefinitionProvider implements vscode.TypeDefinitionProvider {
    provideTypeDefinition(
        document: vscode.TextDocument,
        position: vscode.Position,
        token: vscode.CancellationToken
    ): vscode.Location | undefined {
        // Given a variable, navigate to its type's definition
        return new vscode.Location(
            vscode.Uri.file('/project/src/types.ts'),
            new vscode.Range(10, 0, 25, 1)
        );
    }
}

vscode.languages.registerTypeDefinitionProvider(
    'typescript', new MyTypeDefinitionProvider()
);
```

---

### ImplementationProvider

Navigates to implementations of an interface or abstract method (Go to Implementation).

```typescript
class MyImplementationProvider implements vscode.ImplementationProvider {
    provideImplementation(
        document: vscode.TextDocument,
        position: vscode.Position,
        token: vscode.CancellationToken
    ): vscode.Location[] {
        // Return all classes implementing the interface at the cursor
        return [
            new vscode.Location(
                vscode.Uri.file('/project/src/fileService.ts'),
                new vscode.Position(15, 0)
            ),
            new vscode.Location(
                vscode.Uri.file('/project/src/memoryService.ts'),
                new vscode.Position(8, 0)
            )
        ];
    }
}

vscode.languages.registerImplementationProvider(
    'typescript', new MyImplementationProvider()
);
```

---

### ReferenceProvider

Finds all references to a symbol across the workspace (Find All References).

```typescript
class MyReferenceProvider implements vscode.ReferenceProvider {
    provideReferences(
        document: vscode.TextDocument,
        position: vscode.Position,
        context: vscode.ReferenceContext,
        token: vscode.CancellationToken
    ): vscode.Location[] {
        const includeDeclaration = context.includeDeclaration;
        // Search your index for all usages
        return this.findAllUsages(document, position, includeDeclaration);
    }
}

vscode.languages.registerReferenceProvider('python', new MyReferenceProvider());
```

---

### DocumentSymbolProvider

Provides the outline of symbols in a document (Outline view, Go to Symbol in File).

```typescript
class MyDocumentSymbolProvider implements vscode.DocumentSymbolProvider {
    provideDocumentSymbols(
        document: vscode.TextDocument,
        token: vscode.CancellationToken
    ): vscode.DocumentSymbol[] {
        const symbols: vscode.DocumentSymbol[] = [];

        const classSymbol = new vscode.DocumentSymbol(
            'MyClass',
            'A sample class',
            vscode.SymbolKind.Class,
            new vscode.Range(0, 0, 20, 1),  // full range
            new vscode.Range(0, 6, 0, 13)   // selection range (name)
        );

        const methodSymbol = new vscode.DocumentSymbol(
            'doWork',
            '',
            vscode.SymbolKind.Method,
            new vscode.Range(2, 4, 10, 5),
            new vscode.Range(2, 10, 2, 16)
        );

        classSymbol.children.push(methodSymbol); // Nested hierarchy
        symbols.push(classSymbol);
        return symbols;
    }
}

vscode.languages.registerDocumentSymbolProvider(
    'typescript', new MyDocumentSymbolProvider()
);
```

---

### WorkspaceSymbolProvider

Provides workspace-wide symbol search (Go to Symbol in Workspace, ++ctrl+t++).

```typescript
class MyWorkspaceSymbolProvider implements vscode.WorkspaceSymbolProvider {
    provideWorkspaceSymbols(
        query: string,
        token: vscode.CancellationToken
    ): vscode.SymbolInformation[] {
        // Fuzzy-match query against your symbol index
        return this.symbolIndex
            .filter(s => s.name.toLowerCase().includes(query.toLowerCase()))
            .map(s => new vscode.SymbolInformation(
                s.name,
                s.kind,
                s.containerName,
                new vscode.Location(vscode.Uri.file(s.file), s.range)
            ));
    }
}

vscode.languages.registerWorkspaceSymbolProvider(
    new MyWorkspaceSymbolProvider()
);
```

---

### CodeActionProvider

Provides quick fixes, refactorings, and source actions (lightbulb menu).

```typescript
class MyCodeActionProvider implements vscode.CodeActionProvider {
    static readonly metadata: vscode.CodeActionProviderMetadata = {
        providedCodeActionKinds: [
            vscode.CodeActionKind.QuickFix,
            vscode.CodeActionKind.Refactor
        ]
    };

    provideCodeActions(
        document: vscode.TextDocument,
        range: vscode.Range,
        context: vscode.CodeActionContext,
        token: vscode.CancellationToken
    ): vscode.CodeAction[] {
        const actions: vscode.CodeAction[] = [];

        // Quick fix for diagnostics
        for (const diagnostic of context.diagnostics) {
            if (diagnostic.code === 'missing-import') {
                const fix = new vscode.CodeAction(
                    `Import '${diagnostic.message}'`,
                    vscode.CodeActionKind.QuickFix
                );
                fix.edit = new vscode.WorkspaceEdit();
                fix.edit.insert(
                    document.uri,
                    new vscode.Position(0, 0),
                    `import { ${diagnostic.message} } from 'module';\n`
                );
                fix.diagnostics = [diagnostic];
                fix.isPreferred = true;
                actions.push(fix);
            }
        }

        return actions;
    }
}

vscode.languages.registerCodeActionsProvider(
    'typescript',
    new MyCodeActionProvider(),
    MyCodeActionProvider.metadata
);
```

---

### CodeLensProvider

Displays actionable inline annotations above code (e.g., "Run Test", "5 references").

```typescript
class MyCodeLensProvider implements vscode.CodeLensProvider {
    onDidChangeCodeLenses = new vscode.EventEmitter<void>().event;

    provideCodeLenses(
        document: vscode.TextDocument,
        token: vscode.CancellationToken
    ): vscode.CodeLens[] {
        const lenses: vscode.CodeLens[] = [];
        const text = document.getText();
        const testRegex = /(?:it|test|describe)\s*\(/g;
        let match;

        while ((match = testRegex.exec(text))) {
            const pos = document.positionAt(match.index);
            const range = new vscode.Range(pos, pos);
            lenses.push(new vscode.CodeLens(range));
        }

        return lenses;
    }

    resolveCodeLens(lens: vscode.CodeLens): vscode.CodeLens {
        lens.command = {
            title: 'Run Test',
            command: 'myExtension.runTest',
            arguments: [lens.range.start.line]
        };
        return lens;
    }
}

vscode.languages.registerCodeLensProvider('typescript', new MyCodeLensProvider());
```

---

### DocumentFormattingEditProvider

Formats an entire document (Format Document).

```typescript
class MyFormattingProvider implements vscode.DocumentFormattingEditProvider {
    provideDocumentFormattingEdits(
        document: vscode.TextDocument,
        options: vscode.FormattingOptions,
        token: vscode.CancellationToken
    ): vscode.TextEdit[] {
        const formatted = formatCode(document.getText(), {
            tabSize: options.tabSize,
            insertSpaces: options.insertSpaces
        });

        const fullRange = new vscode.Range(
            document.positionAt(0),
            document.positionAt(document.getText().length)
        );

        return [vscode.TextEdit.replace(fullRange, formatted)];
    }
}

vscode.languages.registerDocumentFormattingEditProvider(
    'python', new MyFormattingProvider()
);
```

---

### DocumentRangeFormattingEditProvider

Formats a selected range within a document (Format Selection).

```typescript
class MyRangeFormattingProvider
    implements vscode.DocumentRangeFormattingEditProvider {
    provideDocumentRangeFormattingEdits(
        document: vscode.TextDocument,
        range: vscode.Range,
        options: vscode.FormattingOptions,
        token: vscode.CancellationToken
    ): vscode.TextEdit[] {
        const text = document.getText(range);
        const formatted = formatCode(text, {
            tabSize: options.tabSize,
            insertSpaces: options.insertSpaces
        });
        return [vscode.TextEdit.replace(range, formatted)];
    }
}

vscode.languages.registerDocumentRangeFormattingEditProvider(
    'python', new MyRangeFormattingProvider()
);
```

---

### OnTypeFormattingEditProvider

Formats code automatically as the user types specific characters (e.g., `;`, `}`, `\n`).

```typescript
class MyOnTypeFormattingProvider
    implements vscode.OnTypeFormattingEditProvider {
    provideOnTypeFormattingEdits(
        document: vscode.TextDocument,
        position: vscode.Position,
        ch: string,
        options: vscode.FormattingOptions,
        token: vscode.CancellationToken
    ): vscode.TextEdit[] {
        if (ch === '}') {
            // Re-indent the closing brace line
            const line = document.lineAt(position.line);
            const indent = computeIndent(document, position.line);
            return [
                vscode.TextEdit.replace(
                    line.range,
                    indent + line.text.trim()
                )
            ];
        }
        return [];
    }
}

vscode.languages.registerOnTypeFormattingEditProvider(
    'javascript',
    new MyOnTypeFormattingProvider(),
    '}', ';', '\n' // trigger characters
);
```

---

### SignatureHelpProvider

Shows parameter hints when the user types a function call.

```typescript
class MySignatureHelpProvider implements vscode.SignatureHelpProvider {
    provideSignatureHelp(
        document: vscode.TextDocument,
        position: vscode.Position,
        token: vscode.CancellationToken,
        context: vscode.SignatureHelpContext
    ): vscode.SignatureHelp {
        const help = new vscode.SignatureHelp();

        const sig = new vscode.SignatureInformation(
            'greet(name: string, greeting?: string): string',
            'Generates a greeting message.'
        );
        sig.parameters = [
            new vscode.ParameterInformation('name: string', 'The name to greet'),
            new vscode.ParameterInformation(
                'greeting?: string',
                'Optional custom greeting prefix'
            )
        ];

        help.signatures = [sig];
        help.activeSignature = 0;
        help.activeParameter = context.activeSignatureHelp?.activeParameter ?? 0;
        return help;
    }
}

vscode.languages.registerSignatureHelpProvider(
    'typescript',
    new MySignatureHelpProvider(),
    { triggerCharacters: ['(', ','], retriggerCharacters: [','] }
);
```

---

### RenameProvider

Renames a symbol across the workspace.

```typescript
class MyRenameProvider implements vscode.RenameProvider {
    provideRenameEdits(
        document: vscode.TextDocument,
        position: vscode.Position,
        newName: string,
        token: vscode.CancellationToken
    ): vscode.WorkspaceEdit {
        const oldName = document.getText(
            document.getWordRangeAtPosition(position)
        );

        const edit = new vscode.WorkspaceEdit();

        // Replace all occurrences across files
        for (const location of this.findAllOccurrences(oldName)) {
            edit.replace(location.uri, location.range, newName);
        }

        return edit;
    }

    prepareRename(
        document: vscode.TextDocument,
        position: vscode.Position
    ): vscode.Range | { range: vscode.Range; placeholder: string } {
        const range = document.getWordRangeAtPosition(position);
        if (!range) {
            throw new Error('Cannot rename this element');
        }
        return { range, placeholder: document.getText(range) };
    }
}

vscode.languages.registerRenameProvider('python', new MyRenameProvider());
```

---

### DocumentHighlightProvider

Highlights all occurrences of the symbol under the cursor within the current document.

```typescript
class MyHighlightProvider implements vscode.DocumentHighlightProvider {
    provideDocumentHighlights(
        document: vscode.TextDocument,
        position: vscode.Position,
        token: vscode.CancellationToken
    ): vscode.DocumentHighlight[] {
        const word = document.getText(
            document.getWordRangeAtPosition(position)
        );
        const highlights: vscode.DocumentHighlight[] = [];
        const text = document.getText();
        const regex = new RegExp(`\\b${word}\\b`, 'g');
        let match;

        while ((match = regex.exec(text))) {
            const start = document.positionAt(match.index);
            const end = document.positionAt(match.index + match[0].length);
            highlights.push(
                new vscode.DocumentHighlight(
                    new vscode.Range(start, end),
                    vscode.DocumentHighlightKind.Read
                )
            );
        }

        return highlights;
    }
}

vscode.languages.registerDocumentHighlightProvider(
    'javascript', new MyHighlightProvider()
);
```

---

### FoldingRangeProvider

Defines foldable regions in a document.

```typescript
class MyFoldingRangeProvider implements vscode.FoldingRangeProvider {
    provideFoldingRanges(
        document: vscode.TextDocument,
        context: vscode.FoldingContext,
        token: vscode.CancellationToken
    ): vscode.FoldingRange[] {
        const ranges: vscode.FoldingRange[] = [];

        // Fold comment blocks
        let commentStart: number | null = null;
        for (let i = 0; i < document.lineCount; i++) {
            const line = document.lineAt(i).text.trim();
            if (line.startsWith('//') && commentStart === null) {
                commentStart = i;
            } else if (!line.startsWith('//') && commentStart !== null) {
                if (i - commentStart > 1) {
                    ranges.push(new vscode.FoldingRange(
                        commentStart, i - 1,
                        vscode.FoldingRangeKind.Comment
                    ));
                }
                commentStart = null;
            }
        }

        return ranges;
    }
}

vscode.languages.registerFoldingRangeProvider(
    'javascript', new MyFoldingRangeProvider()
);
```

---

### SelectionRangeProvider

Provides smart selection ranges for Expand/Shrink Selection (++shift+alt+right++ / ++shift+alt+left++).

```typescript
class MySelectionRangeProvider implements vscode.SelectionRangeProvider {
    provideSelectionRanges(
        document: vscode.TextDocument,
        positions: vscode.Position[],
        token: vscode.CancellationToken
    ): vscode.SelectionRange[] {
        return positions.map(pos => {
            // Innermost: word
            const wordRange = document.getWordRangeAtPosition(pos);
            const word = new vscode.SelectionRange(wordRange!);

            // Next: full line content
            const line = document.lineAt(pos.line);
            const lineRange = new vscode.SelectionRange(
                line.range, word
            );

            // Outermost: entire document
            const fullRange = new vscode.SelectionRange(
                new vscode.Range(0, 0, document.lineCount - 1, 0),
                lineRange
            );

            return fullRange; // Returns the outermost; inner ranges are linked via parent
        });
    }
}

vscode.languages.registerSelectionRangeProvider(
    'typescript', new MySelectionRangeProvider()
);
```

---

### InlayHintsProvider

Displays inline hints for type annotations, parameter names, and other contextual information.

```typescript
class MyInlayHintsProvider implements vscode.InlayHintsProvider {
    onDidChangeInlayHints = new vscode.EventEmitter<void>().event;

    provideInlayHints(
        document: vscode.TextDocument,
        range: vscode.Range,
        token: vscode.CancellationToken
    ): vscode.InlayHint[] {
        const hints: vscode.InlayHint[] = [];

        // Add type annotation hints for variable declarations
        const constRegex = /const\s+(\w+)\s*=/g;
        const text = document.getText(range);
        let match;

        while ((match = constRegex.exec(text))) {
            const pos = document.positionAt(
                document.offsetAt(range.start) + match.index + match[0].length - 1
            );
            const hint = new vscode.InlayHint(
                pos,
                ': string', // inferred type
                vscode.InlayHintKind.Type
            );
            hint.paddingLeft = true;
            hints.push(hint);
        }

        return hints;
    }
}

vscode.languages.registerInlayHintsProvider(
    'typescript', new MyInlayHintsProvider()
);
```

---

### SemanticTokensProvider

Provides semantic token data for rich syntax highlighting beyond TextMate grammars.

```typescript
const tokenTypes = ['class', 'function', 'variable', 'parameter', 'property'];
const tokenModifiers = ['declaration', 'readonly', 'async'];
const legend = new vscode.SemanticTokensLegend(tokenTypes, tokenModifiers);

class MySemanticTokensProvider implements vscode.DocumentSemanticTokensProvider {
    onDidChangeSemanticTokens = new vscode.EventEmitter<void>().event;

    provideDocumentSemanticTokens(
        document: vscode.TextDocument,
        token: vscode.CancellationToken
    ): vscode.SemanticTokens {
        const builder = new vscode.SemanticTokensBuilder(legend);

        // Analyze the document and push tokens
        const analysis = analyzeDocument(document);
        for (const sym of analysis.symbols) {
            builder.push(
                sym.range,
                sym.tokenType,     // e.g., 'function'
                sym.tokenModifiers // e.g., ['async', 'declaration']
            );
        }

        return builder.build();
    }
}

vscode.languages.registerDocumentSemanticTokensProvider(
    'typescript',
    new MySemanticTokensProvider(),
    legend
);
```

---

### LinkedEditingRangeProvider

Enables simultaneous editing of linked ranges (e.g., matching HTML open/close tags).

```typescript
class MyLinkedEditingProvider implements vscode.LinkedEditingRangeProvider {
    provideLinkedEditingRanges(
        document: vscode.TextDocument,
        position: vscode.Position,
        token: vscode.CancellationToken
    ): vscode.LinkedEditingRanges | undefined {
        // Find matching open/close tag
        const tagPair = findMatchingTagPair(document, position);
        if (tagPair) {
            return new vscode.LinkedEditingRanges(
                [tagPair.openTagRange, tagPair.closeTagRange],
                /[a-zA-Z][\w.-]*/ // word pattern for valid tag names
            );
        }
    }
}

vscode.languages.registerLinkedEditingRangeProvider(
    'html', new MyLinkedEditingProvider()
);
```

---

### ColorProvider

Detects colors in a document and provides color pickers for editing them.

```typescript
class MyColorProvider implements vscode.DocumentColorProvider {
    provideDocumentColors(
        document: vscode.TextDocument,
        token: vscode.CancellationToken
    ): vscode.ColorInformation[] {
        const colors: vscode.ColorInformation[] = [];
        const hexRegex = /#([0-9a-fA-F]{6})/g;
        const text = document.getText();
        let match;

        while ((match = hexRegex.exec(text))) {
            const start = document.positionAt(match.index);
            const end = document.positionAt(match.index + match[0].length);
            const r = parseInt(match[1].substr(0, 2), 16) / 255;
            const g = parseInt(match[1].substr(2, 2), 16) / 255;
            const b = parseInt(match[1].substr(4, 2), 16) / 255;

            colors.push(new vscode.ColorInformation(
                new vscode.Range(start, end),
                new vscode.Color(r, g, b, 1)
            ));
        }

        return colors;
    }

    provideColorPresentations(
        color: vscode.Color,
        context: { document: vscode.TextDocument; range: vscode.Range },
        token: vscode.CancellationToken
    ): vscode.ColorPresentation[] {
        const hex = '#' +
            Math.round(color.red * 255).toString(16).padStart(2, '0') +
            Math.round(color.green * 255).toString(16).padStart(2, '0') +
            Math.round(color.blue * 255).toString(16).padStart(2, '0');

        return [new vscode.ColorPresentation(hex)];
    }
}

vscode.languages.registerColorProvider('css', new MyColorProvider());
```

---

### DocumentLinkProvider

Detects clickable links within a document.

```typescript
class MyDocumentLinkProvider implements vscode.DocumentLinkProvider {
    provideDocumentLinks(
        document: vscode.TextDocument,
        token: vscode.CancellationToken
    ): vscode.DocumentLink[] {
        const links: vscode.DocumentLink[] = [];
        const urlRegex = /https?:\/\/[^\s)>]+/g;
        const text = document.getText();
        let match;

        while ((match = urlRegex.exec(text))) {
            const start = document.positionAt(match.index);
            const end = document.positionAt(match.index + match[0].length);
            links.push(new vscode.DocumentLink(
                new vscode.Range(start, end),
                vscode.Uri.parse(match[0])
            ));
        }

        return links;
    }
}

vscode.languages.registerDocumentLinkProvider(
    'markdown', new MyDocumentLinkProvider()
);
```

---

### CallHierarchyProvider

Provides incoming and outgoing call hierarchies for a symbol (Peek Call Hierarchy).

```typescript
class MyCallHierarchyProvider implements vscode.CallHierarchyProvider {
    prepareCallHierarchy(
        document: vscode.TextDocument,
        position: vscode.Position,
        token: vscode.CancellationToken
    ): vscode.CallHierarchyItem | undefined {
        const range = document.getWordRangeAtPosition(position);
        const word = document.getText(range);

        return new vscode.CallHierarchyItem(
            vscode.SymbolKind.Function,
            word,
            '(params)',
            document.uri,
            range!,
            range!
        );
    }

    provideCallHierarchyIncomingCalls(
        item: vscode.CallHierarchyItem,
        token: vscode.CancellationToken
    ): vscode.CallHierarchyIncomingCall[] {
        // Return functions that call `item`
        return this.findCallers(item).map(caller =>
            new vscode.CallHierarchyIncomingCall(caller.item, caller.ranges)
        );
    }

    provideCallHierarchyOutgoingCalls(
        item: vscode.CallHierarchyItem,
        token: vscode.CancellationToken
    ): vscode.CallHierarchyOutgoingCall[] {
        // Return functions that `item` calls
        return this.findCallees(item).map(callee =>
            new vscode.CallHierarchyOutgoingCall(callee.item, callee.ranges)
        );
    }
}

vscode.languages.registerCallHierarchyProvider(
    'typescript', new MyCallHierarchyProvider()
);
```

---

### TypeHierarchyProvider

Provides supertypes and subtypes for a class or interface (Peek Type Hierarchy).

```typescript
class MyTypeHierarchyProvider implements vscode.TypeHierarchyProvider {
    prepareTypeHierarchy(
        document: vscode.TextDocument,
        position: vscode.Position,
        token: vscode.CancellationToken
    ): vscode.TypeHierarchyItem | undefined {
        const range = document.getWordRangeAtPosition(position);
        const word = document.getText(range);

        return new vscode.TypeHierarchyItem(
            vscode.SymbolKind.Class,
            word,
            '',
            document.uri,
            range!,
            range!
        );
    }

    provideTypeHierarchySupertypes(
        item: vscode.TypeHierarchyItem,
        token: vscode.CancellationToken
    ): vscode.TypeHierarchyItem[] {
        // Return parent classes / implemented interfaces
        return this.findSupertypes(item.name);
    }

    provideTypeHierarchySubtypes(
        item: vscode.TypeHierarchyItem,
        token: vscode.CancellationToken
    ): vscode.TypeHierarchyItem[] {
        // Return child classes / implementations
        return this.findSubtypes(item.name);
    }
}

vscode.languages.registerTypeHierarchyProvider(
    'java', new MyTypeHierarchyProvider()
);
```

---

## Registration Pattern Summary

All language providers follow the same registration pattern:

```typescript
const disposable = vscode.languages.register<ProviderType>(
    selector,    // DocumentSelector: language, scheme, pattern
    provider,    // Your provider instance
    ...metadata  // Optional: trigger characters, legend, etc.
);

// Always add to subscriptions for proper cleanup
context.subscriptions.push(disposable);
```

!!! tip "Provider lifecycle"
    - Providers are called by the extension host whenever the user triggers the corresponding feature
    - Return `undefined` or an empty array to indicate "no result"
    - Support `CancellationToken` -- check `token.isCancellationRequested` in long-running operations
    - Providers can return a `Thenable` (Promise) for async operations

---

## Quick Reference Table

| Provider | Registration Method | User Action |
|----------|-------------------|-------------|
| CompletionItemProvider | `registerCompletionItemProvider` | Type / trigger character / ++ctrl+space++ |
| HoverProvider | `registerHoverProvider` | Mouse hover |
| DefinitionProvider | `registerDefinitionProvider` | ++f12++ / ++ctrl+click++ |
| DeclarationProvider | `registerDeclarationProvider` | Go to Declaration |
| TypeDefinitionProvider | `registerTypeDefinitionProvider` | Go to Type Definition |
| ImplementationProvider | `registerImplementationProvider` | Go to Implementation |
| ReferenceProvider | `registerReferenceProvider` | ++shift+f12++ |
| DocumentSymbolProvider | `registerDocumentSymbolProvider` | ++ctrl+shift+o++ |
| WorkspaceSymbolProvider | `registerWorkspaceSymbolProvider` | ++ctrl+t++ |
| CodeActionProvider | `registerCodeActionsProvider` | ++ctrl+period++ / lightbulb |
| CodeLensProvider | `registerCodeLensProvider` | Inline above code |
| DocumentFormattingEditProvider | `registerDocumentFormattingEditProvider` | ++shift+alt+f++ |
| DocumentRangeFormattingEditProvider | `registerDocumentRangeFormattingEditProvider` | Format Selection |
| OnTypeFormattingEditProvider | `registerOnTypeFormattingEditProvider` | Type trigger character |
| SignatureHelpProvider | `registerSignatureHelpProvider` | Type `(` or `,` |
| RenameProvider | `registerRenameProvider` | ++f2++ |
| DocumentHighlightProvider | `registerDocumentHighlightProvider` | Cursor on symbol |
| FoldingRangeProvider | `registerFoldingRangeProvider` | Fold controls in gutter |
| SelectionRangeProvider | `registerSelectionRangeProvider` | ++shift+alt+right++ |
| InlayHintsProvider | `registerInlayHintsProvider` | Inline in editor |
| SemanticTokensProvider | `registerDocumentSemanticTokensProvider` | Syntax highlighting |
| LinkedEditingRangeProvider | `registerLinkedEditingRangeProvider` | Edit matching tag |
| ColorProvider | `registerColorProvider` | Color swatches in editor |
| DocumentLinkProvider | `registerDocumentLinkProvider` | ++ctrl+click++ on link |
| CallHierarchyProvider | `registerCallHierarchyProvider` | Peek Call Hierarchy |
| TypeHierarchyProvider | `registerTypeHierarchyProvider` | Peek Type Hierarchy |
