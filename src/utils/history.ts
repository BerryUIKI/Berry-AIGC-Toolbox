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
  private pending = false;

  private async transition<T>(operation: () => Promise<T>): Promise<T> {
    if (this.pending) throw new Error("An action history transition is already pending");
    this.pending = true;
    try {
      return await operation();
    } finally {
      this.pending = false;
    }
  }

  constructor(maxDepth = 30) {
    this.maxDepth = maxDepth;
  }

  async execute(command: ActionCommand): Promise<void> {
    return this.transition(async () => {
      await command.execute();
      this.undoStack.push(command);
      if (this.undoStack.length > this.maxDepth) {
        this.undoStack.shift();
      }
      this.redoStack = [];
    });
  }

  canUndo(): boolean {
    return !this.pending && this.undoStack.length > 0;
  }

  canRedo(): boolean {
    return !this.pending && this.redoStack.length > 0;
  }

  get lastActionName(): string | null {
    return this.undoStack.length > 0
      ? this.undoStack[this.undoStack.length - 1].name
      : null;
  }

  async undo(): Promise<string | null> {
    return this.transition(async () => {
      const cmd = this.undoStack[this.undoStack.length - 1];
      if (!cmd) return null;
      await cmd.undo();
      this.undoStack.pop();
      this.redoStack.push(cmd);
      return cmd.name;
    });
  }

  async redo(): Promise<string | null> {
    return this.transition(async () => {
      const cmd = this.redoStack[this.redoStack.length - 1];
      if (!cmd) return null;
      if (cmd.redo) {
        await cmd.redo();
      } else {
        await cmd.execute();
      }
      this.redoStack.pop();
      this.undoStack.push(cmd);
      return cmd.name;
    });
  }

  clear(): void {
    if (this.pending) throw new Error("Cannot clear action history during a pending transition");
    this.undoStack = [];
    this.redoStack = [];
  }
}

export const actionHistory = new ActionHistory(50);
