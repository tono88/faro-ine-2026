# Arquitectura

## Flujo

```
INE define SOLICITUD  ->  Institución ENVÍA  ->  VALIDAR  ->  HOMOLOGAR  ->  integrable
   (columnas, tipos,        (CSV hoy;            estructura     catálogo INE     con censos y
    llaves de pareo)         X-Road después)     y contenido    + MEMORIA        encuestas
                                                                    ^
                                                  una persona confirma lo dudoso
```

## Principios

1. **El sistema no adivina.** Solo asigna un código cuando la coincidencia es exacta y única, o
   cuando una persona ya confirmó esa equivalencia. Una cifra oficial no puede depender de un
   parecido de texto.
2. **Lo aprendido no se pierde.** La memoria es un archivo de solo agregar: bitácora auditable y
   compartible entre instituciones.
3. **El contexto manda.** "San José" existe en Escuintla y en Petén. Por eso la solicitud exige
   el nivel superior, y una equivalencia vale solo dentro de su contexto.
4. **Mantenible por el INE.** Solicitudes y catálogos son archivos de texto (TOML y CSV) que un
   analista puede leer y editar sin programar.

## Componentes

| Componente | Tecnología | Estado |
|---|---|---|
| Núcleo: lectura, validación, homologación, memoria, salida estándar | Rust (`faro-homologa`) | Hecho; probado con archivos reales |
| Fuentes: portal CKAN del INE con procedencia | Rust (`faro-fuentes`) | Hecho; sincronización probada contra un portal simulado |
| Territorio: crecimiento de edificaciones, calidad del aire | Rust (`faro-territorio`) | Hecho; falta cargar la serie real de Open Buildings |
| Servidor y API | Rust (`faro-servidor`, axum) | Hecho |
| Base de datos | PostgreSQL 16 | Hecho: equivalencias, bitácora de envíos, pendientes y agregados |
| Pantalla | HTML y JavaScript sin dependencias, servida por el servidor | Hecho (primera versión) |
| Despliegue | Docker Compose | Escrito; pendiente de probar en una máquina con Docker |
| Red vial (OpenStreetMap) y asignación de operadores | — | Pendiente |

La base de datos guarda equivalencias, bitácora y **agregados** (periodo, municipio, sexo). La
salida fila por fila queda en un volumen privado, ya sin nombres ni identificadores directos.

## Relación con X-Road

X-Road es la carretera; FARO es el formulario de aduana.

- X-Road transporta mensajes entre servidores de seguridad de cada institución, con identidad,
  firma y bitácora. No dice qué columnas lleva el mensaje ni cómo se escriben los lugares.
- FARO define el **contrato de contenido** (la solicitud) y verifica que cada envío lo cumpla.
- Una solicitud de FARO puede publicarse como un servicio REST sobre X-Road: la institución
  envía por su servidor de seguridad y el INE valida y homologa al recibir. Mientras una
  institución no esté conectada, el mismo contrato funciona con un archivo CSV.

## Protección de datos

- El repositorio no contiene datos de personas; los ejemplos son ficticios.
- El pareo entre fuentes debe hacerse dentro del INE, con llaves derivadas y sin publicar
  identificadores. FARO no publica microdatos pareados.
- La memoria de equivalencias guarda nombres de lugares, no de personas.
