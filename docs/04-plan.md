# Plan hasta la entrega (jueves 26 nov, 09:00)

> **Este es el plan original, escrito el lunes 23.** Se conserva como constancia. Lo que pasó
> después está al final, en «Cómo terminó».

| Día | Meta | Resultado verificable |
|---|---|---|
| Lun 23 | Repositorio, núcleo de homologación, documentación base | `cargo test` pasa; demo por línea de comandos |
| Mar 24 | Catálogo oficial completo (Censo 2018, lugares poblados); interfaz web para revisar pendientes; llaves de pareo; prueba con un registro administrativo público del INE | Demo de punta a punta con datos reales del INE |
| Mié 25 | Un módulo secundario funcional (prioridad: asignación de operadores, que reutiliza las coordenadas del catálogo); los demás como propuesta documentada; PDF, video y README final | Entregables completos el miércoles por la noche |
| Jue 26 | Revisión final y entrega antes de las 09:00 | Entrega enviada |

## Prioridad si falta tiempo

1. Núcleo funcionando con datos reales del INE.
2. Interfaz de revisión.
3. Asignación de operadores.
4. Crecimiento urbano, red vial y precios: quedan como propuesta con datos y método identificados.

## Cómo terminó

| Lo planeado | Lo hecho |
|---|---|
| Núcleo con datos reales del INE | Hecho el día 1; probado con los archivos reales de RENAP, PNC y SE-CONRED (`06`) |
| Interfaz de revisión | Hecha el día 2; hoy son 24 secciones con cuentas, grupos y recorridos guiados |
| Asignación de operadores | Hecha el día 2 (método húngaro); el día 3 se sumó el trabajo de campo completo |
| Crecimiento urbano, red vial y precios «como propuesta» | Los tres quedaron **funcionando**: GHSL 1975–2020, grafo vial y precios por rubro, de construcción y de referencia |
| PDF, video y README | README, documento técnico, manual gráfico, presentación (PDF y PowerPoint) y clip del video, hechos; falta la narración del video, que graba el equipo (`16`) |
| (No estaba en el plan) Idiomas e inclusión | Agregado el día 3: pantalla en inglés (completa) y en kaqchikel como borrador con términos con fuente; se traduce la interfaz, nunca los datos (`29`) |

Versión por versión: [`08-versiones.md`](08-versiones.md). Conformidad con la nota conceptual:
[`26-conformidad-nota-conceptual.md`](26-conformidad-nota-conceptual.md).

## Pendientes del equipo

- Dar acceso al repositorio a los cinco correos del jurado.
- Enviar la nota de consentimiento firmada.
- ~~Descargar el "Listado de lugares poblados" (XLSX) del INE y agregarlo al proyecto.~~ Hecho el 23-nov: catálogo verificado y lugares poblados cargados con coordenadas.
