# Presentación ante el jurado (7 minutos + 3 de preguntas)

## El mensaje en una frase

**FARO hace que el INE reciba de cualquier institución datos que puede usar el mismo día, sin
adivinar nada, y deja prueba de cada paso.**

## Estructura

| Minuto | Tema | Idea que debe quedar |
|---|---|---|
| 0:00–0:40 | El problema, con las palabras de la Gerencia | No hay un estándar para enviar datos; «Puerto de San José» llega escrito de diez maneras |
| 0:40–2:00 | Demostración 1: pedir, recibir, homologar | No adivina; lo que una persona decide queda aprendido para siempre |
| 2:00–3:10 | Demostración 2: territorio | Del país al lugar poblado; red vial como grafo; operadores de campo |
| 3:10–4:40 | Demostración 3: precios | 85 mil precios reales; código de barras; reglas para diferencias; informe de referencia |
| 4:40–5:20 | Demostración 4: tableros a pedido | Una frase produce un tablero, sin modelos externos |
| 5:20–6:20 | Por qué confiar | Se verifica a sí mismo: estadística, interfaz y seguridad, con informes |
| 6:20–7:00 | Adopción | Un comando; usuarios, roles y grupos; qué sigue |

## Lo que nos distingue (decirlo explícitamente)

1. **Responde a las cuatro necesidades de la Gerencia y a las propuestas de las direcciones**, con el estado de cada una a la vista, incluido lo que no se hizo (`docs/01-necesidades-ine.md`).
2. **Nada inventado**: cada dato tiene fuente, fecha, licencia y huella; cada tabla se regenera con un comando.
3. **Encontró errores en datos oficiales y en sí mismo**: 16 códigos de municipio inexistentes en las proyecciones; y una prueba hacia atrás mostró que su propio margen de proyección acertaba 45 % de las veces cuando prometía 95 %: se corrigió y ahora cumple.
4. **Explicable de principio a fin**: reglas fijas, sin modelos en ejecución; el mismo dato da siempre el mismo resultado.
5. **Listo para usarse**: usuarios, roles, grupos, bitácora, auditoría de seguridad atendida, y funciona en celular.

## Cifras para citar

- 20,036 lugares poblados con población, hogares y viviendas del Censo 2018; 340 municipios; 22 departamentos.
- 707 lugares sin conexión vial mapeada y 83 enlaces probables (51 km).
- 85,907 precios de 18 tiendas; 16 mil productos comparables por código de barras; 3 de cada 4 diferencias grandes son ofertas.
- Tasas de extracción de la FAO que coinciden con la Hoja de Balance 2024 del INE (trigo: 72 %).
- 1,020 proyecciones comprobadas contra lo ocurrido; 6 tamaños de pantalla × 22 secciones sin hallazgos; 20 hallazgos de seguridad, 16 corregidos.

## Preguntas probables y respuesta corta

| Pregunta | Respuesta |
|---|---|
| ¿Usa inteligencia artificial? | En ejecución, no: reglas fijas y a la vista. Hasta los tableros «en lenguaje natural» son reglas que dicen qué entendieron. Se usó un asistente de IA para programar, y está declarado. |
| ¿Y los datos personales? | Los archivos reales se procesan en un temporal que se borra siempre; la salida no lleva nombres ni identificadores; en el repositorio no hay ninguno. |
| ¿Cómo se relaciona con X-Road? | X-Road transporta; FARO define qué se pide y comprueba lo que llega. Se complementan. |
| ¿Qué tan difícil es agregar una institución? | Un archivo de solicitud con sus columnas. Sin programar. |
| ¿Y si se equivoca al homologar? | Solo asigna con coincidencia exacta y única o con una decisión humana registrada, que se puede revertir. |
| ¿Sus proyecciones son confiables? | Lo medimos: al proyectar diez años, 8 de cada 10 veces el error fue de 20 % o menos. Ese es el margen que el portal muestra. |
| ¿Los precios publicados sirven de referencia? | Como antecedente: son fuentes secundarias y el informe lo dice. La regla de la Dirección de Precios pide más de 50 % de cotizaciones directas. |
| ¿Es seguro? | Se auditó completo: sin inyección ni fallos de rol; lo encontrado se corrigió y lo pendiente está escrito. |
| ¿Qué no hace todavía? | Guatecompras (bloquea clientes automáticos), comparación con censos anteriores (no hay fuente descargable), el ICC y el Manual del IPC. |
| ¿Qué necesita el INE para adoptarlo? | Un servidor con Docker, definir sus solicitudes y crear sus usuarios. |
