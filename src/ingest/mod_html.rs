//! Inlined CSS + graph JS for the `trs ingest --html` report. Kept apart from
//! `format_html.rs` so the data-assembly logic stays readable. Both are static
//! strings; `format_html` injects the per-repo data as JS globals ahead of
//! `GRAPH_JS`.

pub(super) const CSS: &str = r####"
:root{
  --bg:#f6f8f8;--surface:#fff;--surface-2:#f0f3f3;--ink:#14181b;--muted:#5a6670;
  --faint:#8a949d;--border:#e2e7e8;--accent:#0d8698;--accent-ink:#0a6975;--accent-soft:#d6ecef;
  --good:#2c9a5c;--warn:#b9781a;--warn-soft:#f4e6cd;--crit:#cf4f49;--crit-soft:#f6dedc;
  --shadow:0 1px 2px rgba(20,24,27,.04),0 8px 24px -12px rgba(20,24,27,.10);
  --mono:ui-monospace,"SF Mono","JetBrains Mono",Menlo,Consolas,monospace;
  --sans:-apple-system,BlinkMacSystemFont,"Segoe UI",Inter,system-ui,sans-serif;
}
@media (prefers-color-scheme:dark){:root{
  --bg:#0d1113;--surface:#161b1e;--surface-2:#1c2327;--ink:#e8edef;--muted:#8c98a1;
  --faint:#606c74;--border:#262e32;--accent:#3fb9cf;--accent-ink:#63c8db;--accent-soft:#123138;
  --good:#46c081;--warn:#e0a838;--warn-soft:#33260f;--crit:#e8756c;--crit-soft:#331a18;
  --shadow:0 1px 2px rgba(0,0,0,.3),0 10px 30px -14px rgba(0,0,0,.55);}}
:root[data-theme="light"]{--bg:#f6f8f8;--surface:#fff;--surface-2:#f0f3f3;--ink:#14181b;
  --muted:#5a6670;--faint:#8a949d;--border:#e2e7e8;--accent:#0d8698;--accent-ink:#0a6975;
  --warn:#b9781a;--warn-soft:#f4e6cd;--good:#2c9a5c;--crit:#cf4f49;
  --shadow:0 1px 2px rgba(20,24,27,.04),0 8px 24px -12px rgba(20,24,27,.10);}
:root[data-theme="dark"]{--bg:#0d1113;--surface:#161b1e;--surface-2:#1c2327;--ink:#e8edef;
  --muted:#8c98a1;--faint:#606c74;--border:#262e32;--accent:#3fb9cf;--accent-ink:#63c8db;
  --warn:#e0a838;--warn-soft:#33260f;--good:#46c081;--crit:#e8756c;
  --shadow:0 1px 2px rgba(0,0,0,.3),0 10px 30px -14px rgba(0,0,0,.55);}
*{box-sizing:border-box}
body{margin:0;background:var(--bg);color:var(--ink);font-family:var(--sans);line-height:1.5;-webkit-font-smoothing:antialiased}
.wrap{max-width:1080px;margin:0 auto;padding:clamp(20px,4vw,52px) clamp(16px,4vw,32px)}
.tnum{font-variant-numeric:tabular-nums}.mono{font-family:var(--mono)}
header{display:flex;flex-wrap:wrap;align-items:flex-end;gap:16px 24px;justify-content:space-between;
  border-bottom:1px solid var(--border);padding-bottom:22px;margin-bottom:32px}
.brand{display:flex;flex-direction:column;gap:6px}
.eyebrow{font-family:var(--mono);font-size:12px;letter-spacing:.14em;text-transform:uppercase;color:var(--accent-ink)}
h1{margin:0;font-size:clamp(26px,5vw,40px);letter-spacing:-.02em;font-weight:700;text-wrap:balance;font-family:var(--mono)}
.sub{color:var(--muted);font-size:15px;max-width:56ch}
.ver{font-family:var(--mono);font-size:13px;color:var(--muted);border:1px solid var(--border);
  border-radius:999px;padding:5px 12px;background:var(--surface);white-space:nowrap}
.kpis{display:grid;grid-template-columns:repeat(auto-fit,minmax(150px,1fr));gap:12px;margin-bottom:34px}
.kpi{background:var(--surface);border:1px solid var(--border);border-radius:12px;padding:16px 18px;box-shadow:var(--shadow)}
.kpi .n{font-family:var(--mono);font-size:26px;font-weight:600;letter-spacing:-.02em;display:block;line-height:1.1}
.kpi .l{font-size:12.5px;color:var(--muted);margin-top:5px}.kpi .l b{color:var(--ink);font-weight:600}
section{margin-bottom:38px}
.h{display:flex;align-items:baseline;gap:12px;margin:0 0 4px}
.h h2{font-size:19px;margin:0;letter-spacing:-.01em}
.h .tag{font-family:var(--mono);font-size:11px;letter-spacing:.1em;text-transform:uppercase;color:var(--faint)}
.lead{color:var(--muted);font-size:14px;margin:0 0 18px;max-width:70ch}
.bars{display:flex;flex-direction:column;gap:9px}
.bar{display:grid;grid-template-columns:230px 1fr 74px;align-items:center;gap:14px}
.bar .name{font-family:var(--mono);font-size:12.5px;color:var(--ink);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.track{height:22px;background:var(--surface-2);border-radius:6px;overflow:hidden;border:1px solid var(--border)}
.fill{height:100%;border-radius:5px 0 0 5px;background:linear-gradient(90deg,var(--accent),color-mix(in oklab,var(--accent),#fff 22%))}
.bar .v{font-family:var(--mono);font-size:12.5px;text-align:right;color:var(--muted)}.bar .v b{color:var(--ink);font-weight:600}
.graph-wrap{position:relative;background:var(--surface);border:1px solid var(--border);border-radius:14px;box-shadow:var(--shadow);overflow:hidden}
#graph{display:block;width:100%;height:460px;cursor:grab;touch-action:none}#graph:active{cursor:grabbing}
.glegend{position:absolute;top:12px;right:13px;display:flex;flex-direction:column;gap:7px;font-family:var(--mono);
  font-size:11px;color:var(--muted);background:color-mix(in oklab,var(--surface),transparent 10%);
  border:1px solid var(--border);border-radius:10px;padding:10px 12px;backdrop-filter:blur(6px)}
.glegend .k{display:flex;align-items:center;gap:8px}.glegend .sw{width:10px;height:10px;border-radius:50%;flex:none}
.ghint{position:absolute;bottom:11px;left:14px;font-family:var(--mono);font-size:11px;color:var(--faint);pointer-events:none}
.card{background:var(--surface);border:1px solid var(--border);border-radius:14px;padding:20px 22px;box-shadow:var(--shadow)}
.card.warn{border-color:color-mix(in oklab,var(--warn),var(--border) 55%)}
.card h3{margin:0 0 12px;font-size:15.5px;display:flex;align-items:center;gap:9px}
.dot{width:9px;height:9px;border-radius:50%;flex:none}.dot.warn{background:var(--warn)}
.rows{display:flex;flex-direction:column;gap:2px}
.row{display:flex;justify-content:space-between;align-items:center;gap:10px;font-family:var(--mono);
  font-size:12.5px;padding:6px 0;border-bottom:1px solid var(--border)}.row:last-child{border-bottom:0}
.row .p{color:var(--ink);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.row .loc{color:var(--muted);white-space:nowrap}
.pill{font-family:var(--mono);font-size:10.5px;letter-spacing:.04em;padding:2px 7px;border-radius:999px;text-transform:uppercase;font-weight:600}
.pill.warn{background:var(--warn-soft);color:var(--warn)}.pill.good{background:var(--accent-soft);color:var(--accent-ink)}
footer{border-top:1px solid var(--border);margin-top:8px;padding-top:20px;color:var(--muted);font-size:13px;
  display:flex;flex-wrap:wrap;gap:8px 18px;justify-content:space-between;align-items:center}
footer code{font-family:var(--mono);background:var(--surface-2);padding:2px 7px;border-radius:6px;color:var(--accent-ink);border:1px solid var(--border)}
@media (max-width:720px){.bar{grid-template-columns:150px 1fr 60px;gap:10px}}
@media (prefers-reduced-motion:no-preference){.fill{animation:grow .9s cubic-bezier(.2,.7,.2,1) both}
  @keyframes grow{from{transform:scaleX(0);transform-origin:left}to{transform:scaleX(1)}}}
"####;

/// Reads globals injected by `format_html`: `BAR_MAX`, `BARS` (`[{name,loc}]`),
/// `GN` (`[{id,loc,deg}]`), `GE` (`[[src,dst]]`).
pub(super) const GRAPH_JS: &str = r####"
(function(){
  var fmt=function(n){return n>=1000?(n/1000).toFixed(1).replace(/\.0$/,'')+'k':''+n;};
  var bc=document.getElementById('bars');
  if(bc){bc.innerHTML=BARS.map(function(m){
    return '<div class="bar"><span class="name" title="'+m.name+'">'+m.name+'</span>'+
      '<div class="track"><div class="fill" style="width:'+Math.max(4,m.loc/BAR_MAX*100)+'%"></div></div>'+
      '<span class="v"><b>'+fmt(m.loc)+'</b></span></div>';}).join('');}

  var cv=document.getElementById('graph');if(!cv||!GN.length)return;
  var ctx=cv.getContext('2d'),hint=document.getElementById('ghint');
  var map=new Map(GN.map(function(n){return [n.id,n];}));
  var maxDeg=GN.reduce(function(m,n){return Math.max(m,n.deg);},1);
  GN.forEach(function(n){n.role=n.deg>=Math.max(4,maxDeg*0.6)?'hub':'mod';});
  var L=GE.map(function(e){return {s:map.get(e[0]),t:map.get(e[1])};}).filter(function(l){return l.s&&l.t;});
  var css=function(k){return getComputedStyle(document.documentElement).getPropertyValue(k).trim();};
  var R=function(n){return Math.max(6,Math.min(26,5+Math.sqrt(Math.max(1,n.loc))/6));};
  var W=0,H=0,dpr=1,inited=false,hover=null,drag=null,alpha=1,running=false;
  function resize(){dpr=Math.min(2,window.devicePixelRatio||1);var r=cv.getBoundingClientRect();
    W=r.width;H=r.height;cv.width=W*dpr;cv.height=H*dpr;ctx.setTransform(dpr,0,0,dpr,0,0);
    if(!inited){var s=Math.min(W,H)*0.33;GN.forEach(function(n,i){var a=i/GN.length*6.283;
      n.x=W/2+Math.cos(a)*s;n.y=H/2+Math.sin(a)*s*0.92;n.vx=0;n.vy=0;});inited=true;}}
  function tick(){
    for(var i=0;i<GN.length;i++)for(var j=i+1;j<GN.length;j++){var a=GN[i],b=GN[j];
      var dx=a.x-b.x,dy=a.y-b.y,d2=dx*dx+dy*dy||0.01,d=Math.sqrt(d2),f=1600/d2,fx=dx/d*f,fy=dy/d*f;
      a.vx+=fx;a.vy+=fy;b.vx-=fx;b.vy-=fy;}
    L.forEach(function(l){var dx=l.t.x-l.s.x,dy=l.t.y-l.s.y,d=Math.sqrt(dx*dx+dy*dy)||0.01,f=(d-92)*0.03,
      fx=dx/d*f,fy=dy/d*f;l.s.vx+=fx;l.s.vy+=fy;l.t.vx-=fx;l.t.vy-=fy;});
    GN.forEach(function(n){n.vx+=(W/2-n.x)*0.006;n.vy+=(H/2-n.y)*0.006;});
    GN.forEach(function(n){if(n===drag)return;n.x+=n.vx*alpha;n.y+=n.vy*alpha;n.vx*=0.82;n.vy*=0.82;
      n.x=Math.max(24,Math.min(W-24,n.x));n.y=Math.max(20,Math.min(H-20,n.y));});
    alpha*=0.97;}
  function neigh(n){return hover&&L.some(function(l){return (l.s===hover&&l.t===n)||(l.t===hover&&l.s===n);});}
  function draw(){ctx.clearRect(0,0,W,H);
    var eC=css('--border'),eH=css('--accent'),ink=css('--ink');
    L.forEach(function(l){var on=hover&&(l.s===hover||l.t===hover);
      ctx.strokeStyle=on?eH:eC;ctx.globalAlpha=hover?(on?0.95:0.12):0.5;ctx.lineWidth=on?1.7:1;
      ctx.beginPath();ctx.moveTo(l.s.x,l.s.y);ctx.lineTo(l.t.x,l.t.y);ctx.stroke();});
    ctx.globalAlpha=1;var cHub=css('--accent'),cMod=css('--muted');
    GN.forEach(function(n){var nb=n===hover||neigh(n),dim=hover&&!nb;
      ctx.globalAlpha=dim?0.22:1;ctx.beginPath();ctx.arc(n.x,n.y,R(n),0,6.2832);
      ctx.fillStyle=n.role==='hub'?cHub:cMod;ctx.fill();
      if(n===hover){ctx.lineWidth=2;ctx.strokeStyle=ink;ctx.stroke();}
      if(n.role==='hub'||nb){ctx.globalAlpha=dim?0.3:1;ctx.fillStyle=ink;
        ctx.font='600 11px ui-monospace,Menlo,monospace';ctx.textAlign='center';
        ctx.fillText(n.id,n.x,n.y-R(n)-5);}});
    ctx.globalAlpha=1;}
  function loop(){tick();draw();if(alpha>0.03||drag){requestAnimationFrame(loop);}else{running=false;alpha=0.02;draw();}}
  function kick(){if(!running){running=true;requestAnimationFrame(loop);}}
  function pick(mx,my){var b=null,bd=1e9;GN.forEach(function(n){var d=Math.hypot(n.x-mx,n.y-my);
    if(d<R(n)+7&&d<bd){bd=d;b=n;}});return b;}
  function xy(e){var r=cv.getBoundingClientRect();var t=e.touches?e.touches[0]:e;return [t.clientX-r.left,t.clientY-r.top];}
  cv.addEventListener('mousemove',function(e){var p=xy(e),mx=p[0],my=p[1];
    if(drag){drag.x=mx;drag.y=my;drag.vx=drag.vy=0;alpha=Math.max(alpha,0.5);kick();return;}
    var h=pick(mx,my);if(h!==hover){hover=h;draw();}
    hint.textContent=hover?(hover.id+'  ·  '+hover.loc.toLocaleString()+' LOC  ·  '+hover.deg+' links'):'hover a node';});
  cv.addEventListener('mousedown',function(e){var p=xy(e);drag=pick(p[0],p[1]);});
  window.addEventListener('mouseup',function(){drag=null;});
  cv.addEventListener('mouseleave',function(){if(!drag){hover=null;hint.textContent='hover a node';draw();}});
  function settle(){for(var i=0;i<340;i++)tick();alpha=0.02;draw();}
  resize();settle();
  var rt;window.addEventListener('resize',function(){clearTimeout(rt);rt=setTimeout(function(){inited=false;resize();settle();},150);});
  var mm=window.matchMedia('(prefers-color-scheme: dark)');
  if(mm.addEventListener)mm.addEventListener('change',draw);
  new MutationObserver(draw).observe(document.documentElement,{attributes:true,attributeFilter:['data-theme']});
})();
"####;
