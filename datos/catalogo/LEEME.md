# Catálogos de lugares

- `departamentos.csv`: 22 departamentos con código INE.
- `municipios.csv`: 340 municipios con código INE (departamento + correlativo) y alias conocidos.
- `lugares_poblados.csv`: 20,036 lugares poblados con código INE (municipio + correlativo de tres
  dígitos) y el centroide de cada uno (`longitud`, `latitud`, grados decimales).

## Estado: verificado contra el Censo 2018

Fuente: INE, XII Censo Nacional de Población y VII de Vivienda 2018, "Listado de lugares poblados"
(conjunto `censo-2018-lugares-poblados` de <https://datos.ine.gob.gt>, licencia CC BY).
Descargado el 23-nov-2026; SHA-256 del archivo
`b8ea8fea33e6d606e102e8d96ed692deacec660c028ffcf673f84459bcafe14e`.

Resultado de la verificación código por código (`faro catalogo verificar`):

- Departamentos: los 22 códigos y nombres coinciden.
- Municipios: los 340 códigos coinciden; ninguno falta ni sobra. En 25 el nombre del catálogo
  provisional era distinto del oficial: 20 usaban el nombre corto (por ejemplo "Nebaj" en vez de
  "Santa María Nebaj") y 5 solo diferían en la grafía. Se adoptó el nombre del listado tal como
  lo publica el INE y el nombre anterior quedó como alias, de modo que ambos se reconocen.
- Lugares poblados: 20,036 códigos, ninguno repetido y todos dentro de su municipio. 19,723 traen
  coordenadas; el listado publica 313 sin centroide: 301 de la categoría "Otros Lugares Poblados"
  (correlativo 999) y otros 12 lugares. Quedan sin coordenadas; no se estiman. Dentro de un mismo municipio hay nombres repetidos (por
  ejemplo varias "Zona 1" de colonias distintas): el sistema los deja como ambiguos y los decide
  una persona.

Los nombres se copian del listado sin corregirlos (incluye, por ejemplo, "Santo Tomas La Unión"
sin tilde).

## Cómo se regenera

```bash
faro fuentes --manifiesto datos/crudo/manifiesto.jsonl descargar censo-2018-lugares-poblados --formato XLSX --recurso "lugares poblados"
faro catalogo verificar --listado datos/crudo/censo-2018-lugares-poblados/<archivo>.xlsx
faro catalogo importar  --listado datos/crudo/censo-2018-lugares-poblados/<archivo>.xlsx
```

`importar` solo escribe si los códigos de departamento y municipio coinciden con el listado.

La columna `alias` lista otros nombres de uso oficial o común, separados por `|`.
