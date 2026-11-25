# Reúne los borradores en kaqchikel, descarta todo rótulo con una palabra que no conste en las
# fuentes y, con --escribir, deja cak.json, cak_revision.csv y cak_palabras.csv en el repositorio.
# Uso: KAQ_TRABAJO=<carpeta de trabajo> python3 armar.py [--escribir]
# En la carpeta de trabajo: tm.json, clk_plano.txt (el diccionario en texto), terminos*.json,
# paq_N.json (paquetes) y kq_N.json (borradores); opcionales cambiar.json y quitar.json.
import json, re, csv, collections, sys, os, glob
S = os.environ.get('KAQ_TRABAJO', 'kaq_trabajo')
R = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..', '..', 'datos', 'estandar', 'idiomas')
ESCRIBIR = '--escribir' in sys.argv
def limpio(t): return re.sub(r'</?\d+/?>|&[a-z]+;|#\d*|\{\d\}', ' ', t.replace('’', "'").replace('‘', "'").replace('`', "'"))
def fichas(t, minus=True):
    out, w = [], ''
    for c in limpio(t) + ' ':
        if c.isalpha() or c == "'": w += c
        else:
            if w.strip("'"): out.append(w.strip("'"))
            w = ''
    return [x.lower() for x in out] if minus else out
tm = json.load(open(S + '/tm.json'))
voc_tm = collections.Counter(w for f in tm for w in fichas(f['cak']))
pl = json.load(open(R + '/plantilla.json'))['textos']
voc_es = set(w for f in tm for w in fichas(f['es'])) | set(w for k in pl for w in fichas(k))
dic = open(S + '/clk_plano.txt', encoding='utf-8').read()
voc_dic = collections.Counter(w for w in fichas(dic))
lex = [t for a in ('terminos.json', 'terminos_extra_interfaz.json') for t in json.load(open(S + '/' + a, encoding='utf-8'))]
voc_lex = set(w for t in lex for w in fichas(t['cak']))
PROPIOS = set('guatemala excel landsat sentinel microsoft forms openstreetmap firefox google docker github'.split())
UNIDADES = set('ha km m kg g lb ml kb mb q'.split())   # unidades y símbolos: iguales en cualquier idioma
SOLO_DICCIONARIO = set('mes'.split())                    # palabras kaqchikel del diccionario que se escriben igual que una española («mes» = basura)
def atestiguada(w):
    if w in UNIDADES: return 'unidad'
    if w in SOLO_DICCIONARIO: return 'almg'
    if voc_tm.get(w, 0) >= 1: return 'mozilla'
    if w in voc_lex: return 'almg'
    if voc_dic.get(w, 0) >= 1 and w not in voc_es: return 'almg'
    return None
def de_la_clave(k):
    ok = set()
    palabras = fichas(k, minus=False)
    for i, w in enumerate(palabras):
        if sum(1 for c in w if c.isupper()) >= 2 or (w[:1].isupper() and i > 0) or w.lower() in PROPIOS or len(w) == 1: ok.add(w.lower())
    return ok
marcas = lambda t: sorted(re.findall(r'</?\d+/?>', t)); ent = lambda t: sorted(re.findall(r'&[a-z]+;', t))
aceptadas, rechazadas = {}, []
for arch in sorted(glob.glob(S + '/kq_*.json')):
    n = re.search(r'kq_(\d+)', arch).group(1)
    entrada = {str(x['id']): x for x in json.load(open(S + f'/paq_{n}.json', encoding='utf-8'))}
    sal = json.load(open(arch, encoding='utf-8'))
    for i, x in entrada.items():
        r = sal.get(i)
        k = x['es']
        if not r or not isinstance(r, dict) or not (r.get('cak') or '').strip(): rechazadas.append((k, '', 'sin respaldo (lo dijo quien redactó)')); continue
        c = r['cak'].replace('’', "'").replace('‘', "'").strip()
        if marcas(c) != marcas(k) or c.count('#') != k.count('#') or ent(c) != ent(k): rechazadas.append((k, c, 'no conserva marcas, números o entidades')); continue
        if c.lower() == k.lower(): rechazadas.append((k, c, 'igual al español')); continue
        permit = de_la_clave(k)
        malas = [w for w in fichas(c) if not atestiguada(w) and w not in permit]
        if malas: rechazadas.append((k, c, 'palabras sin fuente: ' + ', '.join(malas))); continue
        aceptadas[k] = {'cak': c, 'nivel': 'fuente' if r.get('nivel') == 'fuente' else 'compuesto', 'base': r.get('base') or []}
print(len(aceptadas), 'aceptadas de', len(aceptadas) + len(rechazadas), '·', collections.Counter(v['nivel'] for v in aceptadas.values()), f"· {100 * len(aceptadas) / len(pl):.1f} % de {len(pl)}")
print(collections.Counter(r[2].split(':')[0] for r in rechazadas))
json.dump({'aceptadas': aceptadas, 'rechazadas': rechazadas}, open(S + '/resultado.json', 'w', encoding='utf-8'), ensure_ascii=False, indent=1)
if ESCRIBIR:
    quitar = set(json.load(open(S + '/quitar.json', encoding='utf-8'))) if os.path.exists(S + '/quitar.json') else set()
    cambiar = json.load(open(S + '/cambiar.json', encoding='utf-8')) if os.path.exists(S + '/cambiar.json') else {}
    for k, c in cambiar.items():
        if k in aceptadas: aceptadas[k]['cak'] = c
    for k in quitar: aceptadas.pop(k, None)
    d = json.load(open(R + '/cak.json', encoding='utf-8'))
    d['textos'] = {k: (aceptadas[k]['cak'] if k in aceptadas else '') for k in pl}
    json.dump(d, open(R + '/cak.json', 'w', encoding='utf-8'), ensure_ascii=False, indent=1); open(R + '/cak.json', 'a').write('\n')
    with open(R + '/cak_revision.csv', 'w', encoding='utf-8', newline='') as s:
        w = csv.writer(s, lineterminator='\n'); w.writerow(['es', 'cak', 'nivel', 'base', 'revisado_por', 'correccion'])
        for k in pl:
            if k in aceptadas: w.writerow([k, aceptadas[k]['cak'], aceptadas[k]['nivel'], ' | '.join(aceptadas[k]['base'])[:400], '', ''])
    usadas = collections.Counter(w for v in aceptadas.values() for w in fichas(v['cak']))
    ej = {}
    for f in sorted(tm, key=lambda f: len(f['cak'])):
        if not f['es'] or re.search(r'[{<%]', f['cak']): continue
        for w in set(fichas(f['cak'])):
            if w in usadas and w not in ej: ej[w] = (f['cak'], f['es'], f['id'])
    lineas_dic = [l.strip().replace('’', "'") for l in dic.split('\n')]
    with open(R + '/cak_palabras.csv', 'w', encoding='utf-8', newline='') as s:
        w = csv.writer(s, lineterminator='\n'); w.writerow(['palabra', 'fuente', 'veces_en_la_fuente', 'ejemplo_kaqchikel', 'ejemplo_espanol', 'mensaje_o_entrada', 'usos_en_faro'])
        for p in sorted(usadas):
            f = atestiguada(p)
            if f == 'unidad': w.writerow([p, 'unidad de medida (igual en cualquier idioma)', '', '', '', '', usadas[p]])
            elif f == 'mozilla' and p in ej: w.writerow([p, 'Mozilla (Pontoon)', voc_tm[p], ej[p][0][:120], ej[p][1][:120], ej[p][2][:120], usadas[p]])
            elif f:
                l = next((x for x in lineas_dic if p in fichas(x)), '')
                w.writerow([p, 'ALMG-CLK, Kaqchikel Choltzij (2011)', voc_dic.get(p, 0), l[:120], '', 'entrada del diccionario', usadas[p]])
            else: w.writerow([p, 'copiada del español (sigla o nombre propio)', '', '', '', '', usadas[p]])
    print('escrito:', sum(1 for v in d['textos'].values() if v), 'entradas;', len(usadas), 'palabras distintas')
