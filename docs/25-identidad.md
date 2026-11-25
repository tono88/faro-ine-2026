# Identidad de FARO

![FARO · Que cada dato llegue a puerto](identidad/faro-logo.svg)

Proyecto del equipo **La Sociedad de Almas** (Estuardo Sandoval, Ana Echeverría y Alejandro Sandoval)
para el Datatón INE 2026, categoría 2.

## Nombre y lema

**FARO.** Un faro no mueve los barcos ni decide su rumbo: les da una referencia fija para llegar.
FARO hace lo mismo con los datos que cada institución ya tiene.

**Lema: «Que cada dato llegue a puerto.»**

Descriptor, cuando haga falta explicar qué es: *Registros administrativos, territorio y precios.*

## Misión

Dar al INE una forma única, verificable y respetuosa de las personas para pedir, recibir y entender
los registros de otras instituciones, y convertirlos en estadística útil junto al censo, el
territorio y los precios.

## Visión

Un Sistema Estadístico Nacional en el que cada institución entrega sus datos una sola vez, en un
formato común, y el país los aprovecha el mismo día, con la confianza de que nada se adivinó y todo
se puede comprobar.

## Objetivos

1. **Un estándar** para pedir y recibir registros administrativos: la solicitud declarativa.
2. **Homologar sin adivinar** y recordar cada decisión, para no corregir dos veces lo mismo.
3. **Proteger a las personas**: solo agregados, sin datos personales, con conteos pequeños ocultos.
4. **Unir** registros, censo, territorio y precios en un mismo mapa y en tableros publicables.
5. **Trazabilidad**: todo resultado tiene fuente, fecha y un comando que lo repite.
6. **Independencia**: software libre, en un servidor del INE, sin servicios de pago.

Cómo se mide cada uno, con cifras: [`22-antes-y-ahora.md`](22-antes-y-ahora.md).

## Valores

| Valor | Qué significa en la práctica |
|---|---|
| No adivinar | Código oficial solo con coincidencia exacta y única, o con una decisión humana registrada |
| No inventar | Todo dato es real, con fuente, fecha y licencia; lo ficticio se rotula |
| Decir los límites | Cada sección y cada documento dice lo que no se probó o no se pudo verificar |
| Cuidar a las personas | El archivo original no se guarda; las fotos de campo se descartan |

## Logotipo

| Versión | Archivo | Uso |
|---|---|---|
| Horizontal, fondo claro | [`identidad/faro-logo.svg`](identidad/faro-logo.svg) | Documentos, README, presentaciones |
| Horizontal, fondo oscuro | [`identidad/faro-logo-oscuro.svg`](identidad/faro-logo-oscuro.svg) | Portadas y diapositivas azules |
| Icono | [`identidad/faro-icono.svg`](identidad/faro-icono.svg) | Encabezado del portal, pestaña del navegador, redes |

Qué representa: la torre blanca es el catálogo oficial, fijo; las dos franjas son los niveles que
ordena (departamento y municipio); la luz amarilla, a ambos lados, es el dato que sale ya entendido
hacia quien lo necesita.

Reglas de uso:

- Deje alrededor un espacio libre igual a la mitad del alto del icono.
- Tamaño mínimo del icono: 24 px en pantalla, 8 mm impreso.
- No lo deforme, no cambie sus colores ni lo ponga sobre fotos sin un fondo liso.
- **Junto al logotipo del INE, como lo pide su guía.** El INE envió a los equipos su *Guía de diseño
  gráfico para Datatón* (noviembre de 2026). Su logotipo va en el **cintillo inferior**, en la
  esquina inferior derecha, dentro de un recuadro con la esquina superior izquierda curva. FARO lo
  usa así en el pie del menú del portal, la presentación, el manual, el documento técnico y las
  tarjetas del video, con el rótulo «Proyecto participante del Datatón INE 2026»: FARO es una
  propuesta de un equipo participante, no un producto oficial del INE. Los archivos del logotipo
  (versiones permitidas, en color y en negativo) están en [`identidad/ine/`](identidad/ine/); son
  del INE y no se modifican.

## El equipo y su carta de presentación

El equipo se llama **La Sociedad de Almas**. Su emblema es un diseño propio hecho para este
proyecto: un escudo azul oscuro con filete amarillo, tres llamas (una por integrante) y el nombre
dentro. Usa la misma paleta y la misma tipografía que FARO.

![Carta de presentación del equipo](identidad/equipo/carta-de-presentacion.svg)

| Archivo (`docs/identidad/equipo/`) | Qué es |
|---|---|
| `sociedad-de-almas.svg` | El emblema, con el nombre dentro |
| `qr-estuardo-sandoval.svg`, `qr-ana-echeverria.svg`, `qr-alejandro-sandoval.svg` | Código QR al perfil de LinkedIn de cada integrante (comprobados con un lector) |
| `carta-de-presentacion.svg` | Emblema, nombres y los tres códigos en una sola pieza |

La carta cierra cada pieza del proyecto: el README y cada documento (la agrega
`scripts/entregables/pie_equipo.js`, que se puede correr las veces que haga falta), la última
página del manual, del documento técnico y de las preguntas y respuestas, la diapositiva de cierre
y la tarjeta final del video. No lleva fotografías de los integrantes.

## Colores

| Color | Código | Uso |
|---|---|---|
| Azul radiante (tono principal del INE) | `#0080FF` | Acciones, botones, fondo del icono, acentos |
| Azul oscuro (tono principal del INE) | `#004080` | Títulos, menú lateral, texto destacado |
| Blanco (tono principal del INE) | `#FFFFFF` | Fondos |
| Azul claro (el radiante con más luz) | `#66B3FF`, `#B3D9FF`, `#E5F2FF` | Apoyo, franjas, fondos de avisos y filtros, escala del mapa |
| Amarillo (tono secundario del INE) | `#FFD500` | La luz del faro; lo que hay que mirar: recorridos y resaltados |
| Tinta | `#313131` | Texto |
| Gris | `#5b6672` | Texto secundario |

La paleta es la de la *Guía de diseño gráfico para Datatón* del INE: predominan sus tres tonos
principales y el amarillo, que la guía admite como tono secundario, se usa poco, solo donde algo
pide atención. Sustituye al ámbar que FARO usó hasta la v0.20.1 (`#f5a623`), que quedaba fuera de
la guía y se parecía al dorado reservado al Despacho Superior. El gris del texto y los colores de
estado (verde, rojo) no cambiaron.

## Tipografía

**Ubuntu Sans**, la tipografía de la identidad del INE, en todo: 700 para títulos y cifras, 400
para texto. Cifras con ancho fijo en tablas. Va incluida en el portal
(`crates/faro-servidor/web/vendor/ubuntu-sans/`, cuatro pesos reducidos a los caracteres del
español y del kaqchikel) y en los documentos que se generan: no se pide a ningún servicio externo.
Licencia: Ubuntu Font Licence 1.0. Para editar la presentación en PowerPoint hay que tener
instalada Ubuntu Sans (viene en la guía del INE); si no, PowerPoint la sustituye por otra.

## Voz

- Frases cortas, en voz activa, sin jerga: «el sistema no adivina», no «motor de inferencia».
- Primero la cifra, después la explicación.
- Se dice quién hace cada cosa: FARO, el analista, la institución.
- Los límites se escriben con la misma claridad que los logros.

<!-- equipo -->
---

<p align="center"><img src="identidad/equipo/carta-de-presentacion.svg" alt="Equipo La Sociedad de Almas: Estuardo Sandoval, Ana Echeverría y Alejandro Sandoval, con el código QR del perfil de LinkedIn de cada uno" width="760"></p>

<p align="center"><a href="https://www.linkedin.com/in/estuardoantoniosandoval/">Estuardo Sandoval</a> · <a href="https://www.linkedin.com/in/analuciaecheverria/">Ana Echeverría</a> · <a href="https://www.linkedin.com/in/alejandro-jos%C3%A9-sandoval-acevedo-998781aa/">Alejandro Sandoval</a><br>Equipo La Sociedad de Almas · Datatón INE 2026</p>
