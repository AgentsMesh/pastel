import { useState, useCallback } from 'react';
import { Editor } from './components/Editor';
import { Preview } from './components/Preview';
import { NodeTree } from './components/NodeTree';
import { Toolbar } from './components/Toolbar';

const DEFAULT_SOURCE = `canvas "hello-world" {
    width  = 400
    height = 300
    background = #FFFFFF
}

frame main {
    width   = fill
    height  = fill
    layout  = vertical
    align   = center
    justify = center
    gap     = 16

    text "Hello, Pastel!" {
        size   = 32
        weight = bold
        color  = #111111
    }

    text "Design as Code" {
        size  = 16
        color = #666666
    }
}
`;

export function App() {
  const [source, setSource] = useState(DEFAULT_SOURCE);
  const [ir, setIr] = useState<unknown>(null);
  const [error, setError] = useState<string | null>(null);

  const handleSourceChange = useCallback((newSource: string) => {
    setSource(newSource);
  }, []);

  const handleCompiled = useCallback((newIr: unknown) => {
    setIr(newIr);
    setError(null);
  }, []);

  const handleError = useCallback((err: string) => {
    setError(err);
  }, []);

  return (
    <div style={styles.container}>
      <Toolbar source={source} ir={ir} />
      <div style={styles.main}>
        <div style={styles.editorPanel}>
          <div style={styles.panelHeader}>
            <span style={styles.panelTitle}>Source</span>
            <span style={styles.fileTab}>.pastel</span>
          </div>
          <Editor
            source={source}
            onChange={handleSourceChange}
            onCompiled={handleCompiled}
            onError={handleError}
          />
        </div>
        <div style={styles.previewPanel}>
          <div style={styles.panelHeader}>
            <span style={styles.panelTitle}>Preview</span>
            {error && <span style={styles.errorBadge}>Error</span>}
          </div>
          {error ? (
            <div style={styles.errorDisplay}>
              <pre style={styles.errorText}>{error}</pre>
            </div>
          ) : (
            <Preview ir={ir} />
          )}
        </div>
        <div style={styles.sidePanel}>
          <div style={styles.panelHeader}>
            <span style={styles.panelTitle}>Nodes</span>
          </div>
          <NodeTree ir={ir} />
        </div>
      </div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    height: '100%',
  },
  main: {
    display: 'flex',
    flex: 1,
    overflow: 'hidden',
  },
  editorPanel: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    borderRight: '1px solid var(--border)',
    minWidth: 0,
  },
  previewPanel: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    borderRight: '1px solid var(--border)',
    minWidth: 0,
  },
  sidePanel: {
    width: 280,
    display: 'flex',
    flexDirection: 'column',
    flexShrink: 0,
  },
  panelHeader: {
    height: 36,
    display: 'flex',
    alignItems: 'center',
    padding: '0 12px',
    gap: 8,
    borderBottom: '1px solid var(--border)',
    background: 'var(--bg-secondary)',
    flexShrink: 0,
  },
  panelTitle: {
    fontSize: 12,
    fontWeight: 600,
    color: 'var(--text-secondary)',
    textTransform: 'uppercase' as const,
    letterSpacing: '0.05em',
  },
  fileTab: {
    fontSize: 11,
    color: 'var(--text-muted)',
    background: 'var(--bg-surface)',
    padding: '2px 6px',
    borderRadius: 3,
  },
  errorBadge: {
    fontSize: 11,
    color: 'var(--error)',
    background: 'rgba(243, 139, 168, 0.15)',
    padding: '2px 6px',
    borderRadius: 3,
  },
  errorDisplay: {
    flex: 1,
    padding: 16,
    overflow: 'auto',
    background: 'var(--bg-secondary)',
  },
  errorText: {
    fontFamily: 'var(--font-mono)',
    fontSize: 13,
    color: 'var(--error)',
    whiteSpace: 'pre-wrap' as const,
  },
};
