interface NodeTreeProps {
  ir: unknown;
}

export function NodeTree({ ir }: NodeTreeProps) {
  const doc = ir as any;
  if (!doc?.nodes) {
    return <div style={styles.empty}>No nodes</div>;
  }

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        {doc.canvas && (
          <span style={styles.canvas}>
            {doc.canvas.name} ({doc.canvas.width}×{doc.canvas.height})
          </span>
        )}
      </div>
      <div style={styles.tree}>
        {doc.nodes.map((node: any) => (
          <TreeNode key={node.id} node={node} depth={0} />
        ))}
      </div>
    </div>
  );
}

function TreeNode({ node, depth }: { node: any; depth: number }) {
  const indent = depth * 16;
  const typeColors: Record<string, string> = {
    frame: '#89b4fa',
    text: '#a6e3a1',
    image: '#f9e2af',
    shape: '#cba6f7',
  };

  const typeColor = typeColors[node.type] || 'var(--text-primary)';
  const name = node.name || node.id;
  const content = node.props?.content;

  return (
    <>
      <div style={{ ...styles.node, paddingLeft: 8 + indent }}>
        <span style={{ ...styles.type, color: typeColor }}>{node.type}</span>
        <span style={styles.name}>{name}</span>
        {content && (
          <span style={styles.content}>
            &quot;{content.length > 20 ? content.slice(0, 20) + '...' : content}&quot;
          </span>
        )}
      </div>
      {node.children?.map((child: any) => (
        <TreeNode key={child.id} node={child} depth={depth + 1} />
      ))}
    </>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    flex: 1,
    overflow: 'auto',
    fontSize: 12,
    fontFamily: 'var(--font-mono)',
  },
  empty: {
    padding: 16,
    color: 'var(--text-muted)',
    fontSize: 12,
  },
  header: {
    padding: '8px 12px',
    borderBottom: '1px solid var(--border)',
  },
  canvas: {
    color: 'var(--text-secondary)',
    fontSize: 11,
  },
  tree: {
    padding: '4px 0',
  },
  node: {
    display: 'flex',
    alignItems: 'center',
    gap: 6,
    padding: '3px 8px',
    cursor: 'default',
    whiteSpace: 'nowrap' as const,
  },
  type: {
    fontWeight: 600,
    fontSize: 11,
  },
  name: {
    color: 'var(--text-primary)',
  },
  content: {
    color: 'var(--text-muted)',
    fontSize: 11,
    overflow: 'hidden',
    textOverflow: 'ellipsis',
  },
};
