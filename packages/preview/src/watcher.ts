import { watch, type FSWatcher } from 'chokidar';
import { EventEmitter } from 'node:events';
import { resolve, dirname } from 'node:path';

export interface WatchEvent {
  type: 'change' | 'add' | 'unlink';
  path: string;
}

export class PastelWatcher extends EventEmitter {
  private watcher: FSWatcher | null = null;
  private filePath: string;
  private debounceTimer: ReturnType<typeof setTimeout> | null = null;
  private debounceMs: number;

  constructor(filePath: string, debounceMs = 100) {
    super();
    this.filePath = resolve(filePath);
    this.debounceMs = debounceMs;
  }

  /**
   * Start watching the .pastel file and its directory for includes.
   */
  start(): void {
    const dir = dirname(this.filePath);

    this.watcher = watch([this.filePath, `${dir}/**/*.pastel`], {
      ignoreInitial: true,
      awaitWriteFinish: {
        stabilityThreshold: 50,
        pollInterval: 10,
      },
    });

    this.watcher.on('change', (path) => this.handleEvent('change', path));
    this.watcher.on('add', (path) => this.handleEvent('add', path));
    this.watcher.on('unlink', (path) => this.handleEvent('unlink', path));
  }

  /**
   * Stop watching.
   */
  async stop(): Promise<void> {
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
    }
    if (this.watcher) {
      await this.watcher.close();
      this.watcher = null;
    }
  }

  private handleEvent(type: WatchEvent['type'], path: string): void {
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
    }
    this.debounceTimer = setTimeout(() => {
      this.emit('change', { type, path } satisfies WatchEvent);
    }, this.debounceMs);
  }
}
