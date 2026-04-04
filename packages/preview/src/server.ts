import { createServer, type IncomingMessage, type ServerResponse } from 'node:http';
import { resolve } from 'node:path';
import { WebSocketServer, type WebSocket } from 'ws';
import { PastelCompiler, type CompileResult } from './compiler.js';
import { PastelWatcher } from './watcher.js';
import { getPreviewHtml } from './template.js';

export interface PreviewServerOptions {
  filePath: string;
  port?: number;
  pastelBin?: string;
}

export class PastelPreviewServer {
  private options: Required<PreviewServerOptions>;
  private compiler: PastelCompiler;
  private watcher: PastelWatcher;
  private clients: Set<WebSocket> = new Set();
  private lastIr: unknown = null;
  private httpServer: ReturnType<typeof createServer> | null = null;
  private wss: WebSocketServer | null = null;

  constructor(options: PreviewServerOptions) {
    this.options = {
      port: options.port ?? 3210,
      pastelBin: options.pastelBin ?? 'pastel',
      filePath: resolve(options.filePath),
    };
    this.compiler = new PastelCompiler(this.options.pastelBin);
    this.watcher = new PastelWatcher(this.options.filePath);
  }

  async start(): Promise<void> {
    const result = await this.compiler.compile(this.options.filePath);
    this.handleCompileResult(result);

    this.httpServer = createServer((req, res) => this.handleHttp(req, res));

    this.wss = new WebSocketServer({ server: this.httpServer });
    this.wss.on('connection', (ws) => {
      this.clients.add(ws);
      if (this.lastIr) {
        ws.send(JSON.stringify({ type: 'ir', data: this.lastIr }));
      }
      ws.on('close', () => this.clients.delete(ws));
    });

    this.watcher.on('change', async () => {
      const compileResult = await this.compiler.compile(this.options.filePath);
      this.handleCompileResult(compileResult);
      this.broadcast(compileResult);
    });
    this.watcher.start();

    await new Promise<void>((r) => {
      this.httpServer!.listen(this.options.port, () => r());
    });

    console.log(`  ✓ Compiled (${result.durationMs.toFixed(0)}ms)`);
    console.log(`  ✓ Preview: http://localhost:${this.options.port}`);
    console.log(`  ◎ Watching for changes...`);
  }

  async stop(): Promise<void> {
    await this.watcher.stop();
    for (const client of this.clients) {
      client.close();
    }
    this.clients.clear();
    this.wss?.close();
    this.httpServer?.close();
  }

  private handleCompileResult(result: CompileResult): void {
    if (result.success && result.ir) {
      this.lastIr = result.ir;
      console.log(`  ✓ Recompiled (${result.durationMs.toFixed(0)}ms)`);
    } else {
      console.error(`  ✗ Compile error: ${result.error}`);
    }
  }

  private broadcast(result: CompileResult): void {
    const msg = result.success
      ? JSON.stringify({ type: 'ir', data: result.ir })
      : JSON.stringify({ type: 'error', error: result.error });
    for (const client of this.clients) {
      if (client.readyState === 1) client.send(msg);
    }
  }

  private handleHttp(req: IncomingMessage, res: ServerResponse): void {
    if (req.url === '/' || req.url === '/index.html') {
      res.writeHead(200, { 'Content-Type': 'text/html' });
      res.end(getPreviewHtml());
    } else if (req.url === '/api/ir') {
      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify(this.lastIr ?? {}));
    } else {
      res.writeHead(404);
      res.end('Not found');
    }
  }
}
