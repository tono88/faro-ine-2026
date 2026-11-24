# Flujo de proceso

Qué pasa en cada etapa y quién la hace. La pantalla lo muestra en vivo, con las cifras de cada
etapa, en la sección "Proceso". Las etapas marcadas como decisión las toma siempre una persona.

## Registros administrativos: de la solicitud a la estadística

```mermaid
flowchart LR
  subgraph INE["INE: quien pide"]
    A[1. Solicitud de datos<br/>columnas, tipos, obligatorias,<br/>datos personales, valores]
  end
  subgraph INST["Institución"]
    B[2. Envío<br/>Excel o CSV tal como lo tiene]
  end
  subgraph FARO["FARO (automático)"]
    C[3. Lectura y validación<br/>encabezado, columnas, reporte fila por fila]
    D[4. Protección de datos<br/>solo columnas pedidas;<br/>nombres fuera, identificador → llave]
    E[5. Homologación<br/>código oficial solo si es<br/>exacto y único o ya decidido]
    G[7. Memoria<br/>quién, cuándo, de qué institución]
    H[8. Salida estándar<br/>mismo formato, código INE, conteos]
  end
  subgraph ANA["Analista del INE"]
    F{6. Revisión<br/>decide lo que el sistema<br/>no quiso adivinar}
  end
  subgraph USO["Usuarios y portal"]
    I([9. Estadística y mapa<br/>tasas, avance, capas])
  end
  A --> B --> C --> D --> E
  E -- resuelto --> H
  E -- dudoso --> F --> G --> H
  G -. el siguiente envío ya no pregunta .-> E
  H --> I
```

| Etapa | Entrada | Qué hace | Salida | Dónde está |
|---|---|---|---|---|
| 1. Solicitud | Necesidad del INE | Declara columnas, tipos, obligatorias, pareo, dato personal, valores, "sin dato", respaldo | Archivo de solicitud | `datos/solicitudes/*.toml` |
| 2. Envío | Archivo de la institución | Se sube sin reformatear | Archivo en un temporal | Pantalla "Recibir envío" |
| 3. Lectura y validación | Archivo | Ubica encabezado, reconoce alias, valida tipo y contenido | Reporte de problemas | `lectura.rs`, `solicitud.rs` |
| 4. Protección de datos | Filas | Elimina columnas no pedidas y nombres; sustituye identificadores por llave | Filas sin datos personales | `recepcion.rs` |
| 5. Homologación | Textos de lugar y valores | Exacto y único, o memoria; si no, propone | Código o pendiente | `homologar.rs`, `productos.rs` |
| 6. Revisión | Pendientes | Una persona elige | Decisión | Pantalla "Revisar pendientes" |
| 7. Memoria | Decisión | La guarda con autor y fecha | Equivalencia | Base de datos |
| 8. Salida estándar | Filas homologadas | Formato único y conteos por periodo, municipio y sexo | CSV privado y conteos | Volumen privado, base de datos |
| 9. Estadística y mapa | Conteos y población | Tasas, avance, mapa, capas | Capas publicadas | Pantalla y `/capas` |

## Fuentes abiertas: del portal a la capa publicada

```mermaid
flowchart LR
  A[1. Descubrir<br/>catálogo del portal del INE] --> B[2. Descargar con huella<br/>dirección, fecha, licencia, SHA-256]
  B --> C[3. Verificar contra el catálogo<br/>códigos y nombres]
  C -- todo cuadra --> E[5. Derivar la tabla<br/>las partes suman el total]
  C -- algo no cuadra --> D{4. Resolver<br/>coincidencia exacta y única,<br/>o se detiene}
  D --> E --> F([6. Publicar y poder repetir<br/>capa con fuente y licencia])
```

Así se procesaron las proyecciones de población (16 códigos de municipio reubicados), los límites
municipales (dos códigos intercambiados) y la red vial. El caso de población se puede repetir
desde la pantalla ("Población y crecimiento" → "Repetir el proceso ahora") o con `faro poblacion`.

## Precios: de la canasta oficial al precio de referencia

```mermaid
flowchart LR
  A[1. Canasta oficial<br/>artículo, especificación, unidad] --> B[2. Reunir precios<br/>cotizaciones y precios publicados,<br/>con fuente y fecha]
  B --> C[3. Homologar el producto<br/>palabras, medidas, presentación]
  C -- comparable --> E[5. Calcular<br/>un precio por proveedor;<br/>media o mediana]
  C -- por revisar --> D{4. Revisar}
  C -- otra medida --> X[no se compara]
  D --> E --> F([6. Precio de referencia<br/>con fuentes, método y antecedentes])
```
