/**
 * Comments API
 *
 * Code review and commenting support
 */

import { ExtensionHostBridge } from '../bridge';
import { Event, EventEmitter, Disposable } from './events';
import { Uri } from './uri';
import { Range } from './textDocument';

export enum CommentThreadCollapsibleState {
  Collapsed = 0,
  Expanded = 1
}

export enum CommentMode {
  Editing = 0,
  Preview = 1
}

export interface Comment {
  body: string | { value: string; isTrusted?: boolean; supportHtml?: boolean };
  mode: CommentMode;
  author: {
    name: string;
    iconPath?: Uri;
  };
  label?: string;
  timestamp?: Date;
}

export interface CommentThread {
  readonly uri: Uri;
  readonly range: Range;
  comments: readonly Comment[];
  collapsibleState: CommentThreadCollapsibleState;
  canReply: boolean;
  contextValue?: string;
  label?: string;
  state?: any;
  dispose(): void;
}

class CommentThreadImpl implements CommentThread {
  private _comments: readonly Comment[] = [];
  private _collapsibleState = CommentThreadCollapsibleState.Collapsed;
  private _canReply = true;
  private _contextValue?: string;
  private _label?: string;
  private _state?: any;

  constructor(
    private bridge: ExtensionHostBridge,
    private threadId: string,
    public readonly uri: Uri,
    public readonly range: Range
  ) {}

  get comments(): readonly Comment[] {
    return this._comments;
  }

  set comments(value: readonly Comment[]) {
    this._comments = value;
    this.update();
  }

  get collapsibleState(): CommentThreadCollapsibleState {
    return this._collapsibleState;
  }

  set collapsibleState(value: CommentThreadCollapsibleState) {
    this._collapsibleState = value;
    this.update();
  }

  get canReply(): boolean {
    return this._canReply;
  }

  set canReply(value: boolean) {
    this._canReply = value;
    this.update();
  }

  get contextValue(): string | undefined {
    return this._contextValue;
  }

  set contextValue(value: string | undefined) {
    this._contextValue = value;
    this.update();
  }

  get label(): string | undefined {
    return this._label;
  }

  set label(value: string | undefined) {
    this._label = value;
    this.update();
  }

  get state(): any {
    return this._state;
  }

  set state(value: any) {
    this._state = value;
    this.update();
  }

  private update(): void {
    this.bridge.send('updateCommentThread', {
      threadId: this.threadId,
      comments: this._comments,
      collapsibleState: this._collapsibleState,
      canReply: this._canReply,
      contextValue: this._contextValue,
      label: this._label,
      state: this._state
    });
  }

  dispose(): void {
    this.bridge.send('disposeCommentThread', { threadId: this.threadId });
  }
}

export interface CommentingRangeProvider {
  provideCommentingRanges(document: any, token: any): Range[] | Promise<Range[]>;
}

export class CommentController {
  readonly id!: string;
  get label(): string { return ''; }
  set label(value: string) {}
  get options(): any { return undefined; }
  set options(value: any) {}
  get commentingRangeProvider(): CommentingRangeProvider | undefined { return undefined; }
  set commentingRangeProvider(value: CommentingRangeProvider | undefined) {}
  createCommentThread(uri: Uri, range: Range, comments: Comment[]): CommentThread { throw new Error('Not implemented'); }
  dispose(): void {}
}

class CommentControllerImpl extends CommentController {
  private _label: string;
  private _options?: any;
  private _commentingRangeProvider?: CommentingRangeProvider;
  private threads: CommentThread[] = [];

  constructor(
    private bridge: ExtensionHostBridge,
    id: string,
    label: string
  ) {
    super();
    (this as any).id = id;
    this._label = label;
  }

  get label(): string {
    return this._label;
  }

  set label(value: string) {
    this._label = value;
  }

  get options(): any {
    return this._options;
  }

  set options(value: any) {
    this._options = value;
  }

  get commentingRangeProvider(): CommentingRangeProvider | undefined {
    return this._commentingRangeProvider;
  }

  set commentingRangeProvider(value: CommentingRangeProvider | undefined) {
    this._commentingRangeProvider = value;
  }

  createCommentThread(uri: Uri, range: Range, comments: Comment[]): CommentThread {
    const threadId = `thread_${Date.now()}_${Math.random()}`;
    const thread = new CommentThreadImpl(this.bridge, threadId, uri, range);
    thread.comments = comments;
    this.threads.push(thread);

    this.bridge.send('createCommentThread', {
      controllerId: this.id,
      threadId,
      uri,
      range,
      comments
    });

    return thread;
  }

  dispose(): void {
    for (const thread of this.threads) {
      thread.dispose();
    }
    this.bridge.send('disposeCommentController', { id: this.id });
  }
}

export class CommentsAPI {
  constructor(private bridge: ExtensionHostBridge) {}

  createCommentController(id: string, label: string): CommentController {
    return new CommentControllerImpl(this.bridge, id, label);
  }
}
