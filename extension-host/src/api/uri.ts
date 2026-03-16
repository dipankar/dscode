/**
 * Uri API
 *
 * Universal Resource Identifier for files and resources
 */

export class Uri {
  readonly scheme: string;
  readonly authority: string;
  readonly path: string;
  readonly query: string;
  readonly fragment: string;
  readonly fsPath: string;

  private constructor(
    scheme: string,
    authority: string,
    path: string,
    query: string,
    fragment: string
  ) {
    this.scheme = scheme;
    this.authority = authority;
    this.path = path;
    this.query = query;
    this.fragment = fragment;

    // Calculate fsPath
    if (scheme === 'file') {
      // Convert URI path to file system path
      let fsPath = path;
      if (authority && path.startsWith('/')) {
        // UNC path: //authority/path
        fsPath = `//${authority}${path}`;
      } else if (path.length >= 3 && path.charAt(0) === '/' && path.charAt(2) === ':') {
        // Windows drive letter: /c:/path -> c:/path
        fsPath = path.substring(1);
      }
      this.fsPath = fsPath;
    } else {
      this.fsPath = path;
    }
  }

  static file(path: string): Uri {
    return new Uri('file', '', path, '', '');
  }

  static parse(value: string, strict?: boolean): Uri {
    // Simple URI parsing
    const match = value.match(/^(([^:/?#]+):)?(\/\/([^/?#]*))?([^?#]*)(\?([^#]*))?(#(.*))?/);
    if (!match) {
      throw new Error('Invalid URI: ' + value);
    }

    return new Uri(
      match[2] || '',
      match[4] || '',
      match[5] || '',
      match[7] || '',
      match[9] || ''
    );
  }

  static from(components: {
    scheme: string;
    authority?: string;
    path?: string;
    query?: string;
    fragment?: string;
  }): Uri {
    return new Uri(
      components.scheme,
      components.authority || '',
      components.path || '',
      components.query || '',
      components.fragment || ''
    );
  }

  static joinPath(base: Uri, ...pathSegments: string[]): Uri {
    const newPath = [base.path, ...pathSegments].join('/').replace(/\/+/g, '/');
    return new Uri(base.scheme, base.authority, newPath, base.query, base.fragment);
  }

  with(change: {
    scheme?: string;
    authority?: string;
    path?: string;
    query?: string;
    fragment?: string;
  }): Uri {
    return new Uri(
      change.scheme !== undefined ? change.scheme : this.scheme,
      change.authority !== undefined ? change.authority : this.authority,
      change.path !== undefined ? change.path : this.path,
      change.query !== undefined ? change.query : this.query,
      change.fragment !== undefined ? change.fragment : this.fragment
    );
  }

  toString(skipEncoding?: boolean): string {
    let result = '';
    if (this.scheme) {
      result += this.scheme + ':';
    }
    if (this.authority || this.scheme === 'file') {
      result += '//';
    }
    if (this.authority) {
      result += this.authority;
    }
    if (this.path) {
      result += this.path;
    }
    if (this.query) {
      result += '?' + this.query;
    }
    if (this.fragment) {
      result += '#' + this.fragment;
    }
    return result;
  }

  toJSON(): any {
    return {
      scheme: this.scheme,
      authority: this.authority,
      path: this.path,
      query: this.query,
      fragment: this.fragment,
      fsPath: this.fsPath,
      $mid: 1
    };
  }
}
