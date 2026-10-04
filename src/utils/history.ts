export interface ActionCommand {
  name: string;
  execute(): Promise<void>;
  undo(): Promise<void>;
  redo?(): Promise<void>;
}

export class ActionHistory {
  private undoStack: ActionCommand[] = [];
  private redoStack: ActionCommand[] = [];
  private maxDepth: number;

  constructor(maxDepth = 30) {
    this.maxDepth = maxDepth;
  }

  async execute(command: ActionCommand): Promise<void> {
    await command.execute();
    this.undoStack.push(command);
    if (this.undoStack.length > this.maxDepth) {
      this.undoStack.shift();
    }
    this.redoStack = [];
  }

  canUndo(): boolean {
    return this.undoStack.length > 0;
  }

  canRedo(): boolean {
    return this.redoStack.length > 0;
  }

  get lastActionName(): string | null {
    return this.undoStack.length > 0
      ? this.undoStack[this.undoStack.length - 1].name
      : null;
  }

  async undo(): Promise<string | null> {
    const cmd = this.undoStack.pop();
    if (!cmd) return null;
    await cmd.undo();
    this.redoStack.push(cmd);
    return cmd.name;
  }

  async redo(): Promise<string | null> {
    const cmd = this.redoStack.pop();
    if (!cmd) return null;
    if (cmd.redo) {
      await cmd.redo();
    } else {
      await cmd.execute();
    }
    this.undoStack.push(cmd);
    return cmd.name;
  }

  clear(): void {
    this.undoStack = [];
    this.redoStack = [];
  }
}

export const actionHistory = new ActionHistory(50);
