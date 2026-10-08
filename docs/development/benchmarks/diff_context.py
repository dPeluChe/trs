import importlib.util,subprocess,os,sys,statistics
spec=importlib.util.spec_from_file_location("truth","docs/development/benchmarks/truth.py"); t=importlib.util.module_from_spec(spec); spec.loader.exec_module(t)
name,count=t.tokenizer()
T=os.path.abspath("target/release/trs")
commits=subprocess.run(["git","log","--no-merges","-60","--format=%h"],capture_output=True,text=True).stdout.split()
rows=[]
for c in commits:
    raw=subprocess.run(["git","diff",f"{c}~1",c,"--","src","docs","tests/cli_gradle_real_output.rs"],capture_output=True,text=True,errors="replace").stdout
    if not raw.strip(): continue
    changed=sum(1 for l in raw.splitlines() if l[:1] in "+-" and l[:3] not in ("+++","---"))
    if changed<4 or changed>500: continue
    rows.append((c,raw,changed))
print(tokenizer:=name, "diffs:",len(rows))
for ctx in (0,1,2,3):
    env=dict(os.environ,TRS_DIFF_CONTEXT=str(ctx))
    tot_raw=tot_out=0; kept=[]; 
    for c,raw,ch in rows:
        out=subprocess.run([T,"git","diff",f"{c}~1",c,"--","src","docs","tests/cli_gradle_real_output.rs"],capture_output=True,text=True,errors="replace",env=env).stdout
        a=t.anchors(raw)
        tot_raw+=count(raw); tot_out+=count(out)
        if a: kept.append(t.kept(a,out)/len(a))
    print(f"context={ctx}: tokens cut {100*(tot_raw-tot_out)/tot_raw:5.1f}%   anchors kept median {100*statistics.median(kept):5.1f}%  mean {100*statistics.mean(kept):5.1f}%")
