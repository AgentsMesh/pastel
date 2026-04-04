import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { existsSync } from 'node:fs';

const execFileAsync = promisify(execFile);

export interface CompileResult {
  success: boolean;
  ir?: unknown;
  error?: string;
  durationMs: number;
}

export class PastelCompiler {
  private pastelBin: string;

  constructor(pastelBin?: string) {
    this.pastelBin = pastelBin ?? 'pastel';
  }

  /**
   * Compile a .pastel file to IR JSON.
   */
  async compile(filePath: string): Promise<CompileResult> {
    const start = performance.now();

    if (!existsSync(filePath)) {
      return {
        success: false,
        error: `File not found: ${filePath}`,
        durationMs: performance.now() - start,
      };
    }

    try {
      const { stdout, stderr } = await execFileAsync(this.pastelBin, [
        'inspect',
        filePath,
        '--json',
      ]);

      const ir = JSON.parse(stdout);
      return {
        success: true,
        ir,
        durationMs: performance.now() - start,
      };
    } catch (err: unknown) {
      const error = err as { stderr?: string; message?: string };
      return {
        success: false,
        error: error.stderr || error.message || 'Unknown compilation error',
        durationMs: performance.now() - start,
      };
    }
  }

  /**
   * Validate a .pastel file without generating IR.
   */
  async check(filePath: string): Promise<CompileResult> {
    const start = performance.now();

    try {
      const { stdout } = await execFileAsync(this.pastelBin, ['check', filePath]);
      return {
        success: true,
        durationMs: performance.now() - start,
      };
    } catch (err: unknown) {
      const error = err as { stderr?: string; message?: string };
      return {
        success: false,
        error: error.stderr || error.message || 'Unknown error',
        durationMs: performance.now() - start,
      };
    }
  }
}
