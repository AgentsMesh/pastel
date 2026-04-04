/**
 * Returns the preview page HTML with embedded Canvas 2D renderer.
 * This is served by the preview server at /.
 */
export function getPreviewHtml(): string {
  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Pastel Preview</title>
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body { font-family: -apple-system, sans-serif; background: #1a1a2e;
      display: flex; flex-direction: column; align-items: center; min-height: 100vh; padding: 24px; }
    .header { color: #e0e0e0; font-size: 13px; margin-bottom: 16px; display: flex; align-items: center; gap: 8px; }
    .status { width: 8px; height: 8px; border-radius: 50%; background: #4caf50; }
    .status.error { background: #f44336; }
    .status.connecting { background: #ff9800; }
    #canvas-container { background: white; border-radius: 8px; box-shadow: 0 4px 24px rgba(0,0,0,0.3); overflow: hidden; }
    canvas { display: block; }
    .error-overlay { position: fixed; bottom: 24px; left: 50%; transform: translateX(-50%);
      background: #f44336; color: white; padding: 12px 24px; border-radius: 8px;
      font-family: monospace; font-size: 13px; display: none; max-width: 80%; white-space: pre-wrap; }
    .node-tree { color: #ccc; font-family: monospace; font-size: 12px; margin-top: 16px;
      background: #16213e; padding: 16px; border-radius: 8px; max-width: 100%; overflow-x: auto; }
  </style>
</head>
<body>
  <div class="header"><div class="status" id="status"></div><span id="status-text">Connecting...</span></div>
  <div id="canvas-container"><canvas id="canvas"></canvas></div>
  <div class="error-overlay" id="error"></div>
  <pre class="node-tree" id="tree"></pre>
  <script>${getClientScript()}</script>
</body>
</html>`;
}

function getClientScript(): string {
  return `
    const canvas = document.getElementById('canvas');
    const ctx = canvas.getContext('2d');
    const statusEl = document.getElementById('status');
    const statusText = document.getElementById('status-text');
    const errorEl = document.getElementById('error');
    const treeEl = document.getElementById('tree');

    function connect() {
      const ws = new WebSocket('ws://' + location.host);
      ws.onopen = () => { statusEl.className = 'status'; statusText.textContent = 'Connected'; errorEl.style.display = 'none'; };
      ws.onmessage = (e) => {
        const msg = JSON.parse(e.data);
        if (msg.type === 'ir') { renderDesign(msg.data); errorEl.style.display = 'none'; }
        else if (msg.type === 'error') { errorEl.textContent = msg.error; errorEl.style.display = 'block'; }
      };
      ws.onclose = () => { statusEl.className = 'status connecting'; statusText.textContent = 'Reconnecting...'; setTimeout(connect, 1000); };
    }
    connect();

    function renderDesign(ir) {
      if (!ir || !ir.canvas) return;
      const { width, height, background } = ir.canvas;
      canvas.width = width; canvas.height = height;
      canvas.style.width = width + 'px'; canvas.style.height = height + 'px';
      ctx.clearRect(0, 0, width, height);
      if (background) { ctx.fillStyle = background; ctx.fillRect(0, 0, width, height); }
      const map = {};
      layoutNodes(ir.nodes, 0, 0, width, height, null, [0,0,0,0], map);
      ir.nodes.forEach(n => renderNode(n, map));
      treeEl.textContent = formatTree(ir.nodes, '');
      statusText.textContent = 'Connected — ' + ir.canvas.name + ' (' + width + 'x' + height + ')';
    }

    function layoutNodes(nodes, px, py, pw, ph, layout, padding, map) {
      if (!nodes || !nodes.length) return;
      const mode = layout?.mode || 'vertical';
      const gap = layout?.gap || 0;
      const p = padding || [0,0,0,0];
      const ix = px+p[3], iy = py+p[0], iw = pw-p[1]-p[3], ih = ph-p[0]-p[2];
      const isH = mode === 'horizontal';
      const sizes = nodes.map(n => ({ w: dimVal(n.width, iw), h: dimVal(n.height, ih) }));
      sizes.forEach((s,i) => { const n = nodes[i]; if (n.type==='text'&&n.content) { const fs=n.font_size||14; if(!s.w)s.w=n.content.length*fs*0.6; if(!s.h)s.h=fs*1.4; } });
      const totalGap = gap * Math.max(0, nodes.length-1);
      const total = sizes.reduce((s,d) => s + (isH?d.w:d.h), 0) + totalGap;
      const free = (isH?iw:ih) - total;
      const justify = layout?.justify || 'start';
      const align = layout?.align || 'start';
      let cx = ix, cy = iy;
      if (justify==='center') { if(isH)cx+=free/2; else cy+=free/2; }
      if (justify==='end') { if(isH)cx+=free; else cy+=free; }
      let spaceBetween = 0;
      if (justify==='space-between' && nodes.length>1) spaceBetween = free/(nodes.length-1);

      nodes.forEach((node, i) => {
        let {w,h} = sizes[i]; let nx=cx, ny=cy;
        if(isH) { if(align==='center')ny=iy+(ih-h)/2; if(align==='end')ny=iy+ih-h; }
        else { if(align==='center')nx=ix+(iw-w)/2; if(align==='end')nx=ix+iw-w; }
        map[node.id] = {x:nx,y:ny,w,h};
        if(node.children?.length) layoutNodes(node.children,nx,ny,w,h,node.layout||null,node.padding||[0,0,0,0],map);
        if(isH) cx+=w+gap+spaceBetween; else cy+=h+gap+spaceBetween;
      });
    }

    function dimVal(dim, ps) {
      if(!dim) return 0;
      if(typeof dim==='number') return dim;
      if(dim.type==='number') return dim.value;
      if(dim.type==='fill') return ps;
      return 0;
    }

    function renderNode(node, map) {
      const r = map[node.id]; if(!r) return;
      const {x,y,w,h} = r;
      if(node.shadow) { ctx.save(); ctx.shadowColor=node.shadow.color||'transparent'; ctx.shadowBlur=node.shadow.blur||0; ctx.shadowOffsetX=node.shadow.x||0; ctx.shadowOffsetY=node.shadow.y||0; }
      if(node.fill?.type==='solid') { ctx.fillStyle=node.fill.color; if(node.corner_radius) { roundRect(ctx,x,y,w,h,node.corner_radius); ctx.fill(); } else ctx.fillRect(x,y,w,h); }
      if(node.shadow) ctx.restore();
      if(node.stroke) { ctx.strokeStyle=node.stroke.color; ctx.lineWidth=node.stroke.width; ctx.strokeRect(x,y,w,h); }
      if(node.type==='text'&&node.content) { const fs=node.font_size||14; ctx.font=(node.font_weight||'normal')+' '+fs+'px -apple-system,sans-serif'; ctx.fillStyle=node.color||'#000'; ctx.textBaseline='top'; const ta=node.text_align||'left'; let tx=x; if(ta==='center'){ctx.textAlign='center';tx=x+w/2;} else ctx.textAlign='left'; ctx.fillText(node.content,tx,y); }
      if(node.type==='image'&&w>0&&h>0) { ctx.fillStyle='#f0f0f0'; ctx.fillRect(x,y,w,h); ctx.strokeStyle='#ddd'; ctx.lineWidth=1; ctx.beginPath();ctx.moveTo(x,y);ctx.lineTo(x+w,y+h);ctx.stroke(); ctx.beginPath();ctx.moveTo(x+w,y);ctx.lineTo(x,y+h);ctx.stroke(); ctx.fillStyle='#999'; ctx.font='11px sans-serif'; ctx.textAlign='center'; ctx.textBaseline='middle'; ctx.fillText(node.asset||node.name||'img',x+w/2,y+h/2); }
      if(node.children) node.children.forEach(c => renderNode(c, map));
    }

    function roundRect(ctx,x,y,w,h,r) {
      if(Array.isArray(r)) { const [tl,tr,br,bl]=r; ctx.beginPath(); ctx.moveTo(x+tl,y); ctx.lineTo(x+w-tr,y); ctx.quadraticCurveTo(x+w,y,x+w,y+tr); ctx.lineTo(x+w,y+h-br); ctx.quadraticCurveTo(x+w,y+h,x+w-br,y+h); ctx.lineTo(x+bl,y+h); ctx.quadraticCurveTo(x,y+h,x,y+h-bl); ctx.lineTo(x,y+tl); ctx.quadraticCurveTo(x,y,x+tl,y); ctx.closePath(); }
    }

    function formatTree(nodes, prefix) {
      if(!nodes) return '';
      let out = '';
      nodes.forEach((n,i) => {
        const last = i===nodes.length-1;
        const conn = last?'└── ':'├── ';
        const content = n.content?' "'+n.content.slice(0,30)+'"':'';
        out += prefix+conn+n.type+(n.name?' '+n.name:'')+content+'\\n';
        if(n.children?.length) out += formatTree(n.children, prefix+(last?'    ':'│   '));
      });
      return out;
    }
  `;
}
