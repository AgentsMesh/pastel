import { describe, it, expect } from 'vitest';
import { PastelCompiler } from '../src/compiler.js';
import { resolve } from 'node:path';

const WORKSPACE_ROOT = resolve(import.meta.dirname, '../../..');
const PASTEL_BIN = resolve(WORKSPACE_ROOT, 'target/debug/pastel');

describe('PastelCompiler', () => {
  const compiler = new PastelCompiler(PASTEL_BIN);

  it('should compile hello-world example', async () => {
    const result = await compiler.compile(
      resolve(WORKSPACE_ROOT, 'examples/hello-world/main.pastel')
    );
    expect(result.success).toBe(true);
    expect(result.ir).toBeDefined();
    expect((result.ir as any).canvas.name).toBe('hello-world');
    expect((result.ir as any).nodes).toHaveLength(1);
    expect(result.durationMs).toBeGreaterThan(0);
  });

  it('should compile landing-page example', async () => {
    const result = await compiler.compile(
      resolve(WORKSPACE_ROOT, 'examples/landing-page/main.pastel')
    );
    expect(result.success).toBe(true);
    expect((result.ir as any).canvas.name).toBe('landing-page');
    expect((result.ir as any).canvas.width).toBe(1440);
    expect((result.ir as any).assets).toHaveLength(2);
    expect((result.ir as any).nodes).toHaveLength(2);
  });

  it('should fail on nonexistent file', async () => {
    const result = await compiler.compile('/nonexistent/file.pastel');
    expect(result.success).toBe(false);
    expect(result.error).toBeDefined();
  });

  it('should check a valid file', async () => {
    const result = await compiler.check(
      resolve(WORKSPACE_ROOT, 'examples/hello-world/main.pastel')
    );
    expect(result.success).toBe(true);
  });
});
