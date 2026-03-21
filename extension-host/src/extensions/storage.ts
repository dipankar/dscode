import * as fs from 'fs';
import * as path from 'path';

export class PersistentMemento {
  private data: Record<string, any> = {};

  constructor(private readonly filePath: string) {
    this.ensureDirectory();
    this.load();
  }

  get<T>(key: string, defaultValue?: T): T | undefined {
    if (Object.prototype.hasOwnProperty.call(this.data, key)) {
      return this.data[key] as T;
    }
    return defaultValue;
  }

  keys(): readonly string[] {
    return Object.keys(this.data);
  }

  async update<T>(key: string, value: T | undefined): Promise<void> {
    if (value === undefined) {
      delete this.data[key];
    } else {
      this.data[key] = value;
    }
    await this.save();
  }

  private ensureDirectory() {
    const dir = path.dirname(this.filePath);
    fs.mkdirSync(dir, { recursive: true });
  }

  private load() {
    try {
      if (fs.existsSync(this.filePath)) {
        const content = fs.readFileSync(this.filePath, 'utf-8');
        this.data = JSON.parse(content);
      }
    } catch (error) {
      console.error('[Storage] Failed to load memento', error);
      this.data = {};
    }
  }

  private async save() {
    try {
      await fs.promises.writeFile(this.filePath, JSON.stringify(this.data, null, 2), 'utf-8');
    } catch (error) {
      console.error('[Storage] Failed to persist memento', error);
    }
  }
}
