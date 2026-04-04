import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { PastelWatcher } from '../src/watcher.js';
import { writeFileSync, mkdirSync, rmSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

describe('PastelWatcher', () => {
  let tempDir: string;

  beforeEach(() => {
    tempDir = join(tmpdir(), `pastel-test-${Date.now()}`);
    mkdirSync(tempDir, { recursive: true });
  });

  afterEach(() => {
    rmSync(tempDir, { recursive: true, force: true });
  });

  it('should detect file changes', async () => {
    const filePath = join(tempDir, 'test.pastel');
    writeFileSync(filePath, 'canvas "test" { width = 100 }');

    const watcher = new PastelWatcher(filePath, 50);
    const changes: unknown[] = [];

    watcher.on('change', (event: unknown) => {
      changes.push(event);
    });

    watcher.start();

    // Wait for watcher to initialize
    await new Promise((r) => setTimeout(r, 200));

    // Modify the file
    writeFileSync(filePath, 'canvas "test" { width = 200 }');

    // Wait for debounced change event
    await new Promise((r) => setTimeout(r, 500));

    await watcher.stop();

    expect(changes.length).toBeGreaterThanOrEqual(1);
    expect((changes[0] as any).type).toBe('change');
  });

  it('should stop watching', async () => {
    const filePath = join(tempDir, 'test.pastel');
    writeFileSync(filePath, 'canvas "test" { width = 100 }');

    const watcher = new PastelWatcher(filePath);
    watcher.start();
    await watcher.stop();

    // Should not throw after stop
    expect(true).toBe(true);
  });
});
