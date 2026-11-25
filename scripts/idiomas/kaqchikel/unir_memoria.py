# Une las memorias de traducción de Pontoon (cak y es-MX) por identificador de mensaje.
# Uso: python3 unir_memoria.py <carpeta con tm_cak.tmx y tm_es-MX.tmx> <salida tm.json>
# Las dos memorias se bajan de https://pontoon.mozilla.org/translation-memory/<idioma>.all-projects.tmx
import sys, re, json, html, collections
d = sys.argv[1]
def leer(ruta, lang):
    t = open(ruta, encoding='utf-8').read()
    out = {}
    for m in re.finditer(r'<tu tuid="([^"]*)"[^>]*>\s*<tuv xml:lang="en-US">\s*<seg>(.*?)</seg>\s*</tuv>\s*<tuv xml:lang="%s">\s*<seg>(.*?)</seg>' % re.escape(lang), t, re.S):
        out[m.group(1)] = (html.unescape(m.group(2)).strip(), html.unescape(m.group(3)).strip())
    return out
cak, es = leer(d + '/tm_cak.tmx', 'cak'), leer(d + '/tm_es-MX.tmx', 'es-MX')
filas = []
for k, (en, c) in cak.items():
    e = es.get(k)
    filas.append({'id': k, 'en': en, 'cak': c, 'es': e[1] if e and e[0] == en else ''})
json.dump(filas, open(sys.argv[2], 'w', encoding='utf-8'), ensure_ascii=False)
con = [f for f in filas if f['es']]
print(len(filas), 'unidades en kaqchikel;', len(con), 'con español alineado')
print(collections.Counter(f['id'].split(':')[0] for f in filas).most_common(25))
