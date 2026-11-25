# Para cada rótulo de FARO arma un paquete de evidencia: frases cortas de la memoria de Mozilla
# (español -> kaqchikel) que contienen cada palabra de contenido del rótulo, más las entradas del
# diccionario de la ALMG-CLK y los términos ya comprobados.
# Uso: python3 paquetes.py <tm.json> <plantilla.json> <salida paquetes.json> <largo máximo del rótulo>
#          <diccionario en texto: pdftotext Diccionario-2-Kaqchikel.pdf> [terminos.json ...]
import sys, re, json, collections, unicodedata
tm = json.load(open(sys.argv[1])); pl = json.load(open(sys.argv[2]))['textos']; salida = sys.argv[3]
VACIAS = set('de del la el los las un una unos unas y o u a al en con por para sin se su sus que es son lo le les no si ya más muy este esta estos estas ese esa eso como cada entre sobre desde hasta ante the of e'.split())
def llano(t): return ''.join(c for c in unicodedata.normalize('NFD', t.lower()) if unicodedata.category(c) != 'Mn')
def palabras(t): return [w for w in re.findall(r"[a-záéíóúñü]+", re.sub(r'<[^>]*>|&[a-z]+;|\{\d\}', ' ', t.lower())) if w not in VACIAS and len(w) > 1]
def raiz(w):
    w = llano(w)
    for s in ('ciones', 'cion', 'es', 's'):
        if len(w) > len(s) + 3 and w.endswith(s): return w[:-len(s)] + ('cion' if s in ('ciones',) else '')
    return w
indice = collections.defaultdict(list)
limpio = lambda c: not re.search(r'[{<%$&\\]|\n', c)
for f in tm:
    if not f['es'] or not limpio(f['cak']) or not limpio(f['es']): continue
    ws = palabras(f['es'])
    if 0 < len(ws) <= 5 and len(f['es']) <= 60:
        for w in set(ws): indice[raiz(w)].append((len(ws), f['es'], f['cak'], f['en']))
exacto = collections.defaultdict(collections.Counter); ing = {}
norm = lambda x: re.sub(r'[…:.!¡¿?]+$', '', re.sub(r'\s+', ' ', x.strip())).strip().lower()
for f in tm:
    if f['es'] and limpio(f['cak']): exacto[norm(f['es'])][f['cak']] += 1; ing.setdefault((norm(f['es']), f['cak']), f['en'])
# Diccionario de la Comunidad Lingüística Kaqchikel (ALMG), parte castellano => kaqchikel, y términos reunidos de otras fuentes.
dic = collections.defaultdict(list)
lineas = open(sys.argv[5], encoding='utf-8').read().split(chr(10))
ini = next(i for i, l in enumerate(lineas) if 'KAXLAN ⇒ KAQCHIKEL' in l)
cuerpo = [l.strip().replace('’', "'") for l in lineas[ini + 1:]]
for i, l in enumerate(cuerpo):
    m = re.match(r"^([a-záéíóúñü]+)[ ,(]", l.lower() + ' ')
    if not m or len(l) < 4 or l.isdigit(): continue
    entrada = l
    if (l.endswith(',') or l.endswith('-') or len(l.split()) < 2) and i + 1 < len(cuerpo): entrada += ' ' + cuerpo[i + 1]
    dic[raiz(m.group(1))].append(entrada)
lex = collections.defaultdict(list)
for arch in sys.argv[6:]:
    for t in json.load(open(arch, encoding='utf-8')):
        lex[raiz(t['es'].split()[0].lower())].append(f"{t['es']} => {t['cak'].replace('’', chr(39))}  (fuente: {t['fuente']}{'; ' + t['nota'] if t.get('nota') else ''})"[:260])
paquetes = []
for i, k in enumerate(pl):
    if '{' in k or '|' in k or len(k) > int(sys.argv[4]): continue
    ws = palabras(k)
    if not ws: continue
    ev = {}
    for w in dict.fromkeys(ws):
        c = collections.Counter(); ej = {}
        for n, es, cak, en in indice.get(raiz(w), []):
            c[(es.strip(), cak.strip())] += 1; ej[(es.strip(), cak.strip())] = (n, en)
        orden = sorted(c, key=lambda p: (ej[p][0], -c[p], len(p[0])))[:5]
        ev[w] = [f"{es} => {cak}  [{ej[(es, cak)][1]}] x{c[(es, cak)]}" for es, cak in orden]
        d = sorted(set(dic.get(raiz(w), [])), key=len)[:6]
        if d: ev[w] += ['DICCIONARIO ALMG-CLK 2011: ' + x for x in d]
        if lex.get(raiz(w)): ev[w] += ['LEXICO: ' + x for x in lex[raiz(w)][:4]]
    ex = [f"{a}  [{ing[(norm(k), a)]}] x{b}" for a, b in exacto.get(norm(k), collections.Counter()).most_common(3)]
    paquetes.append({'id': i, 'es': k, 'identico': ex, 'evidencia': ev})
json.dump(paquetes, open(salida, 'w', encoding='utf-8'), ensure_ascii=False, indent=1)
sin = sum(1 for p in paquetes if any(not v for v in p['evidencia'].values()))
print(len(paquetes), 'rótulos con paquete;', sin, 'con alguna palabra sin evidencia')
falt = collections.Counter(w for p in paquetes for w, v in p['evidencia'].items() if not v)
print('palabras sin evidencia más frecuentes:', falt.most_common(70))
