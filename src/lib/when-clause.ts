import type { CommandContext } from './command-context';

type Token =
  | { type: 'identifier'; value: string }
  | { type: 'string'; value: string }
  | { type: 'boolean'; value: boolean }
  | { type: 'not' }
  | { type: 'and' }
  | { type: 'or' }
  | { type: 'eq' }
  | { type: 'neq' }
  | { type: 'lparen' }
  | { type: 'rparen' };

export function evaluateWhenClause(
  expression: string | undefined,
  context: CommandContext,
): boolean {
  if (!expression?.trim()) {
    return true;
  }

  try {
    const parser = new WhenClauseParser(tokenize(expression), context);
    return parser.parse();
  } catch (error) {
    console.warn('[WhenClause] Falling back to permissive evaluation for:', expression, error);
    return true;
  }
}

function tokenize(input: string): Token[] {
  const tokens: Token[] = [];
  let index = 0;

  while (index < input.length) {
    const char = input[index];

    if (/\s/.test(char)) {
      index += 1;
      continue;
    }

    if (char === '(') {
      tokens.push({ type: 'lparen' });
      index += 1;
      continue;
    }

    if (char === ')') {
      tokens.push({ type: 'rparen' });
      index += 1;
      continue;
    }

    if (char === '!' && input[index + 1] === '=') {
      tokens.push({ type: 'neq' });
      index += 2;
      continue;
    }

    if (char === '!') {
      tokens.push({ type: 'not' });
      index += 1;
      continue;
    }

    if (char === '&' && input[index + 1] === '&') {
      tokens.push({ type: 'and' });
      index += 2;
      continue;
    }

    if (char === '|' && input[index + 1] === '|') {
      tokens.push({ type: 'or' });
      index += 2;
      continue;
    }

    if (char === '=' && input[index + 1] === '=') {
      tokens.push({ type: 'eq' });
      index += 2;
      continue;
    }

    if (char === '\'' || char === '"') {
      const quote = char;
      index += 1;
      let value = '';

      while (index < input.length && input[index] !== quote) {
        value += input[index];
        index += 1;
      }

      if (input[index] === quote) {
        index += 1;
      }

      tokens.push({ type: 'string', value });
      continue;
    }

    let value = '';
    while (index < input.length && /[A-Za-z0-9._:$-]/.test(input[index])) {
      value += input[index];
      index += 1;
    }

    if (!value) {
      throw new Error(`Unexpected token near "${input.slice(index)}"`);
    }

    if (value === 'true' || value === 'false') {
      tokens.push({ type: 'boolean', value: value === 'true' });
    } else {
      tokens.push({ type: 'identifier', value });
    }
  }

  return tokens;
}

class WhenClauseParser {
  private index = 0;

  constructor(
    private readonly tokens: Token[],
    private readonly context: CommandContext,
  ) {}

  parse(): boolean {
    const value = this.parseOr();
    if (this.peek()) {
      throw new Error(`Unexpected trailing token: ${JSON.stringify(this.peek())}`);
    }
    return value;
  }

  private parseOr(): boolean {
    let value = this.parseAnd();

    while (this.match('or')) {
      const right = this.parseAnd();
      value = value || right;
    }

    return value;
  }

  private parseAnd(): boolean {
    let value = this.parseUnary();

    while (this.match('and')) {
      const right = this.parseUnary();
      value = value && right;
    }

    return value;
  }

  private parseUnary(): boolean {
    if (this.match('not')) {
      return !this.parseUnary();
    }

    return this.parsePrimary();
  }

  private parsePrimary(): boolean {
    if (this.match('lparen')) {
      const value = this.parseOr();
      this.consume('rparen');
      return value;
    }

    const token = this.advance();
    if (!token) {
      throw new Error('Unexpected end of when-clause');
    }

    if (token.type === 'boolean') {
      return token.value;
    }

    if (token.type === 'identifier' || token.type === 'string') {
      const leftValue = this.resolveValue(token);
      const operator = this.peek();

      if (operator?.type === 'eq' || operator?.type === 'neq') {
        this.advance();
        const rightToken = this.advance();
        if (!rightToken) {
          throw new Error('Missing comparison value');
        }

        const rightValue = this.resolveComparisonValue(rightToken);
        return operator.type === 'eq'
          ? leftValue === rightValue
          : leftValue !== rightValue;
      }

      return toBoolean(leftValue);
    }

    throw new Error(`Unexpected token type: ${token.type}`);
  }

  private resolveValue(token: Token): string | boolean | undefined {
    if (token.type === 'string') {
      return token.value;
    }

    if (token.type === 'boolean') {
      return token.value;
    }

    if (token.type !== 'identifier') {
      return undefined;
    }

    switch (token.value) {
      case 'activeEditor':
        return this.context.activeEditor;
      case 'editorFocus':
        return this.context.editorFocus;
      case 'editorTextFocus':
        return this.context.editorTextFocus;
      case 'editorHasSelection':
        return this.context.editorHasSelection;
      case 'explorerViewletFocus':
        return this.context.explorerViewletFocus;
      case 'searchViewletFocus':
        return this.context.searchViewletFocus;
      case 'scmViewletFocus':
        return this.context.scmViewletFocus;
      case 'sideBarFocus':
        return this.context.sideBarFocus;
      case 'resourceExtname':
        return this.context.resourceExtname;
      case 'activeEditorLangId':
      case 'editorLangId':
      case 'resourceLangId':
        return this.context.activeEditorLangId;
      default:
        return this.context.custom?.[token.value] as string | boolean | undefined;
    }
  }

  private resolveComparisonValue(token: Token): string | boolean | undefined {
    const value = this.resolveValue(token);

    if (value !== undefined || token.type !== 'identifier') {
      return value;
    }

    // VS Code-style when clauses often use bare literals like `.rs`.
    return token.value;
  }

  private match(type: Token['type']): boolean {
    if (this.peek()?.type === type) {
      this.index += 1;
      return true;
    }

    return false;
  }

  private consume(type: Token['type']) {
    if (!this.match(type)) {
      throw new Error(`Expected token ${type}`);
    }
  }

  private peek() {
    return this.tokens[this.index];
  }

  private advance() {
    const token = this.tokens[this.index];
    this.index += 1;
    return token;
  }
}

function toBoolean(value: string | boolean | undefined): boolean {
  if (typeof value === 'boolean') {
    return value;
  }

  return Boolean(value);
}
