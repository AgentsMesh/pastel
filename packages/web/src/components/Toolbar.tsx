interface ToolbarProps {
  source: string;
  ir: unknown;
}

export function Toolbar({ source, ir }: ToolbarProps) {
  const handleExportPng = () => {
    const canvas = document.querySelector('canvas');
    if (!canvas) return;
    canvas.toBlob((blob) => {
      if (!blob) return;
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'design.png';
      a.click();
      URL.revokeObjectURL(url);
    });
  };

  const handleExportJson = () => {
    if (!ir) return;
    const json = JSON.stringify(ir, null, 2);
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'design.ir.json';
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleExportSource = () => {
    const blob = new Blob([source], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'design.pastel';
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div style={styles.toolbar}>
      <div style={styles.brand}>
        <span style={styles.logo}>Pastel</span>
        <span style={styles.subtitle}>Design as Code</span>
      </div>
      <div style={styles.actions}>
        <button style={styles.button} onClick={handleExportSource}>
          Save .pastel
        </button>
        <button style={styles.button} onClick={handleExportJson}>
          Export IR
        </button>
        <button style={{ ...styles.button, ...styles.primaryButton }} onClick={handleExportPng}>
          Export PNG
        </button>
      </div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  toolbar: {
    height: 48,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: '0 16px',
    borderBottom: '1px solid var(--border)',
    background: 'var(--bg-secondary)',
    flexShrink: 0,
  },
  brand: {
    display: 'flex',
    alignItems: 'center',
    gap: 8,
  },
  logo: {
    fontSize: 16,
    fontWeight: 700,
    color: 'var(--accent)',
    letterSpacing: '-0.02em',
  },
  subtitle: {
    fontSize: 11,
    color: 'var(--text-muted)',
  },
  actions: {
    display: 'flex',
    gap: 8,
  },
  button: {
    height: 30,
    padding: '0 12px',
    fontSize: 12,
    fontWeight: 500,
    border: '1px solid var(--border)',
    borderRadius: 6,
    background: 'var(--bg-surface)',
    color: 'var(--text-secondary)',
    cursor: 'pointer',
    fontFamily: 'inherit',
  },
  primaryButton: {
    background: 'var(--accent)',
    color: '#1e1e2e',
    borderColor: 'var(--accent)',
  },
};
