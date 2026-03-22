/**
 * Tests for Uri API
 */

import { Uri } from '../api/uri';

describe('Uri', () => {
  describe('static file', () => {
    it('should create Uri from file path', () => {
      const uri = Uri.file('/home/user/document.txt');

      expect(uri.scheme).toBe('file');
      expect(uri.fsPath).toBe('/home/user/document.txt');
      expect(uri.path).toBe('/home/user/document.txt');
    });

    it('should handle Windows-style paths', () => {
      const uri = Uri.file('C:\\Users\\user\\document.txt');

      expect(uri.scheme).toBe('file');
      expect(uri.fsPath).toBe('C:\\Users\\user\\document.txt');
    });

    it('should handle paths with spaces', () => {
      const uri = Uri.file('/home/user/my documents/file.txt');

      expect(uri.fsPath).toBe('/home/user/my documents/file.txt');
    });

    it('should handle paths with special characters', () => {
      const uri = Uri.file('/home/user/file-with_special.chars.txt');

      expect(uri.fsPath).toBe('/home/user/file-with_special.chars.txt');
    });
  });

  describe('static parse', () => {
    it('should parse file URI string', () => {
      const uri = Uri.parse('file:///home/user/document.txt');

      expect(uri.scheme).toBe('file');
      expect(uri.path).toBe('/home/user/document.txt');
    });

    it('should parse http URI string', () => {
      const uri = Uri.parse('https://example.com/path/to/resource');

      expect(uri.scheme).toBe('https');
      expect(uri.authority).toBe('example.com');
      expect(uri.path).toBe('/path/to/resource');
    });

    it('should parse URI with query string', () => {
      const uri = Uri.parse('https://example.com/path?foo=bar&baz=qux');

      expect(uri.query).toBe('foo=bar&baz=qux');
    });

    it('should parse URI with fragment', () => {
      const uri = Uri.parse('https://example.com/path#section');

      expect(uri.fragment).toBe('section');
    });

    it('should parse vscode-specific schemes', () => {
      const uri = Uri.parse('vscode-resource:/extension/path/icon.png');

      expect(uri.scheme).toBe('vscode-resource');
    });
  });

  describe('toString', () => {
    it('should serialize file URI', () => {
      const uri = Uri.file('/home/user/document.txt');
      const str = uri.toString();

      expect(str).toBe('file:///home/user/document.txt');
    });

    it('should serialize http URI', () => {
      const uri = Uri.parse('https://example.com/path');
      const str = uri.toString();

      expect(str).toBe('https://example.com/path');
    });

    it('should preserve path characters in toString', () => {
      const uri = Uri.file('/home/user/file with spaces.txt');

      // Uri.file doesn't encode the path
      const str = uri.toString();
      expect(str).toBe('file:///home/user/file with spaces.txt');
    });
  });

  describe('with method', () => {
    it('should create new Uri with modified scheme', () => {
      const original = Uri.file('/path/to/file.txt');
      const modified = original.with({ scheme: 'untitled' });

      expect(modified.scheme).toBe('untitled');
      expect(modified.path).toBe(original.path);
      expect(original.scheme).toBe('file'); // Original unchanged
    });

    it('should create new Uri with modified path', () => {
      const original = Uri.file('/path/to/file.txt');
      const modified = original.with({ path: '/new/path.txt' });

      expect(modified.path).toBe('/new/path.txt');
      expect(original.path).toBe('/path/to/file.txt'); // Original unchanged
    });

    it('should create new Uri with modified query', () => {
      const original = Uri.parse('https://example.com/path');
      const modified = original.with({ query: 'key=value' });

      expect(modified.query).toBe('key=value');
    });

    it('should create new Uri with modified fragment', () => {
      const original = Uri.parse('https://example.com/path');
      const modified = original.with({ fragment: 'section' });

      expect(modified.fragment).toBe('section');
    });
  });

  describe('fsPath', () => {
    it('should return path for file URIs', () => {
      const uri = Uri.file('/home/user/file.txt');

      expect(uri.fsPath).toBe('/home/user/file.txt');
    });

    it('should preserve encoded characters in path', () => {
      const uri = Uri.parse('file:///home/user/file%20with%20spaces.txt');

      // Uri.parse doesn't decode URL-encoded characters
      expect(uri.path).toBe('/home/user/file%20with%20spaces.txt');
    });
  });

  describe('joinPath', () => {
    it('should join path segments', () => {
      const base = Uri.file('/home/user');
      const joined = Uri.joinPath(base, 'documents', 'file.txt');

      expect(joined.fsPath).toBe('/home/user/documents/file.txt');
    });

    it('should handle single path segment', () => {
      const base = Uri.file('/home/user');
      const joined = Uri.joinPath(base, 'file.txt');

      expect(joined.fsPath).toBe('/home/user/file.txt');
    });

    it('should handle empty path segments', () => {
      const base = Uri.file('/home/user');
      const joined = Uri.joinPath(base);

      expect(joined.fsPath).toBe('/home/user');
    });
  });

  describe('toJSON', () => {
    it('should serialize Uri to JSON', () => {
      const uri = Uri.parse('https://example.com/path?query=value#fragment');
      const json = uri.toJSON();

      expect(json).toEqual({
        $mid: 1,
        scheme: 'https',
        authority: 'example.com',
        path: '/path',
        query: 'query=value',
        fragment: 'fragment',
        fsPath: '/path'
      });
    });
  });
});
