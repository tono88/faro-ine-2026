# Clasificador de fotos: ¿vivienda o comercio?

Al registrar una visita de campo, el operador puede tomar una foto del lugar. El portal dice si la
foto parece una **vivienda**, un **comercio** u **otra cosa**. Si parece un comercio, la visita se
marca para que alguien la revise. El resultado nunca rechaza una visita por sí solo.

## Qué modelo es

- **Places365-ResNet18**, de MIT CSAIL: reconoce 365 tipos de escena («house», «shopfront»,
  «market/outdoor»…). Es un modelo liviano: 45 MB y 11 millones de parámetros.
- **FARO no lo entrenó ni lo ajustó.** Lo usa como lo publicaron sus autores. Los pesos tienen
  licencia Creative Commons Atribución (CC BY) y el código, MIT.
- **Corre dentro del servidor**, en Rust (`tract`), sin servicios externos ni tarjeta gráfica. En
  la máquina de pruebas tarda menos de un segundo por foto.
- Procedencia, huellas y cómo se exportó: [`datos/modelos/PROCEDENCIA.md`](../datos/modelos/PROCEDENCIA.md).

## Cómo decide

1. El modelo da una probabilidad para cada una de las 365 escenas.
2. FARO suma las de 41 escenas que son vivienda (casa, barrio residencial, patio, cocina…) y las de
   53 que son comercio (tienda, mercado, restaurante, farmacia…). Las listas están en
   `crates/faro-fuentes/src/fachada.rs` y se eligieron leyendo las 365 categorías, antes de probar.
3. Si las dos sumas juntas no llegan a 25 %, la foto es de **otra cosa** (una calle, una iglesia,
   un paisaje) y no se dice nada sobre la visita.
4. Si llegan, gana la mayor. Su **confianza** es la parte que se lleva de las dos. Con menos de
   65 % el resultado se informa como **no concluyente** y tampoco cuenta.

Qué pasa después, en el cruce de la visita (`crates/faro-servidor/src/campo.rs`):

| Resultado de la foto | Efecto en la visita |
|---|---|
| Vivienda, concluyente | Cuenta a favor |
| Comercio, concluyente | La visita se marca «conviene revisar» |
| No concluyente u otra cosa | Queda como nota; no pone en duda la visita |

## Qué se hace con la foto

- El teléfono la reduce a 640 píxeles de lado antes de enviarla. Así pesa poco y no lleva los
  datos de la cámara ni la posición que guarda el archivo original.
- El servidor la clasifica en memoria y **la descarta**. En la base queda la clase, la confianza y
  las tres escenas más probables.

## Qué tan bien funciona

**Prueba con fotos de fachadas** (`faro fachadas`): 16 fotos de Wikimedia Commons, 14 de Guatemala
y 2 de Honduras, ocho de viviendas y ocho de comercios, elegidas antes de correr el modelo.

| Resultado | Fotos |
|---|---|
| Acierto | 11 |
| Error (dice vivienda donde hay comercio, o al revés) | 0 |
| No concluyente | 5 |

| Esperado | Foto | Resultado | Confianza | Escena más probable | Veredicto | Autor y licencia de la foto |
|---|---|---|---|---|---|---|
| vivienda | cantel casa | vivienda | 81 % | slum 19 % | acierto | [Simon Burchell](https://commons.wikimedia.org/wiki/File:Cantel_4.jpg), CC BY-SA 3.0 |
| vivienda | jacaltenango casa en construccion | otro | 91 % | arch 19 % | no concluyente | [camposeco](https://commons.wikimedia.org/wiki/File:CASA_EN_CONSTRCCION_-_panoramio.jpg), CC BY 3.0 |
| vivienda | mixco colonia residencial | vivienda | 88 % | beach_house 15 % | acierto | [Luis-Ro Martinez](https://commons.wikimedia.org/wiki/File:Ca%C3%B1adas_de_Encinal_Z.7_Mixco_-_panoramio.jpg), CC BY-SA 3.0 |
| vivienda | antigua casa esquina amarilla | otro | 84 % | church/outdoor 12 % | no concluyente | [Bruno Rijsman](https://commons.wikimedia.org/wiki/File:Antigua_(Mexico,_February_2020)_-_110_(49987776488).jpg), CC BY-SA 2.0 |
| vivienda | antigua casa roja | vivienda | 58 % | motel 22 % | no concluyente | [Bruno Rijsman](https://commons.wikimedia.org/wiki/File:Antigua_(Mexico,_February_2020)_-_44_(49987640188).jpg), CC BY-SA 2.0 |
| vivienda | antigua casa 2004 | vivienda | 81 % | chalet 16 % | acierto | [LBM1948](https://commons.wikimedia.org/wiki/File:Antigua_Guatemala_(2004)_11.jpg), CC BY-SA 4.0 |
| vivienda | ojojona casa | vivienda | 99 % | cottage 32 % | acierto | [chamo estudio](https://commons.wikimedia.org/wiki/File:Ojojona_Honduras_house.jpg), CC BY 2.0 |
| vivienda | santa lucia casa colonial | comercio | 56 % | inn/outdoor 41 % | no concluyente | [Luis Alfredo Romero](https://commons.wikimedia.org/wiki/File:Colonial_style_house_in_Santa_Lucia.jpg), CC BY-SA 4.0 |
| comercio | jocotenango tienda | comercio | 100 % | shopfront 39 % | acierto | [Mario Sánchez Prada](https://commons.wikimedia.org/wiki/File:Antigua_Guatemala_2016_-_04.jpg), CC BY-SA 2.0 |
| comercio | chimaltenango agroservicio | otro | 97 % | barndoor 62 % | no concluyente | [LBM1948](https://commons.wikimedia.org/wiki/File:Chimaltenango_(2004).jpg), CC BY-SA 4.0 |
| comercio | mercado zonal | comercio | 100 % | market/outdoor 80 % | acierto | [Iro Bosero](https://commons.wikimedia.org/wiki/File:Guatemala_2024_04.jpg), CC BY-SA 4.0 |
| comercio | xela restaurante | comercio | 99 % | general_store/outdoor 40 % | acierto | [John Barrie](https://commons.wikimedia.org/wiki/File:Alka-Seltzer_Restaurante_NanooK.jpg), CC BY 2.0 |
| comercio | ipala restaurante | comercio | 100 % | diner/outdoor 64 % | acierto | [Ever Giron](https://commons.wikimedia.org/wiki/File:El_legendario_restaurante_Chevere_frente_al_palacio_municipal_de_Ipala..jpg), CC BY-SA 3.0 |
| comercio | asuncion mita farmacia | comercio | 99 % | general_store/outdoor 85 % | acierto | [Daniel chavez castro](https://commons.wikimedia.org/wiki/File:Drugstore,_original_building_Aug07_-_panoramio.jpg), CC BY 3.0 |
| comercio | antigua tienda | comercio | 89 % | general_store/outdoor 65 % | acierto | [Callie Reed](https://commons.wikimedia.org/wiki/File:Tienda,_Antigua,_Guatemala.jpg), CC BY 2.0 |
| comercio | quetzaltenango tienda | comercio | 100 % | ticket_booth 65 % | acierto | [Ken Mayer](https://commons.wikimedia.org/wiki/File:Tienda_Sendero_Luminoso.jpg), CC BY 2.0 |

Las fotos no están en el repositorio (cada una tiene su licencia); la tabla completa, con el enlace
de cada foto, está en [`datos/referencia/fachadas_prueba.csv`](../datos/referencia/fachadas_prueba.csv).
Repetir: `faro fachadas --fotos <carpeta con subcarpetas vivienda/ y comercio/>`.

**Sobre el conjunto de validación de los autores** (36,500 imágenes de todo el mundo): el modelo
acierta la escena exacta en 53.6 % de las imágenes y la tiene entre sus cinco primeras en 83.7 %.
Sumando por grupo, como hace FARO, acierta el grupo en 93 % de las imágenes de vivienda y en 93.5 %
de las de comercio.

## Límites, dichos con claridad

- **Dieciséis fotos son una prueba de humo, no una validación.** Los umbrales (25 % y 65 %) se
  fijaron mirando esas mismas fotos. Antes de un uso real hay que medirlo con fotos de campo del INE.
- **No conoce la arquitectura local.** Una casa colonial de Antigua le parece una iglesia o un
  hostal, y una casa de bloque con arcos, un arco. Por eso lo dudoso no cuenta en contra.
- **Una vivienda con tienda al frente** (caso frecuente) saldrá como comercio. La marca «conviene
  revisar» es correcta ahí: alguien debe decidir.
- **No reconoce personas ni lee rótulos.** Solo clasifica el tipo de escena.
- **Los pesos se bajaron de una copia** en GitHub, porque el sitio de los autores no respondió
  desde la nube de trabajo. Se comprobó que es el mismo archivo (detalle en la procedencia), pero
  conviene bajarlo del sitio oficial desde la red del INE y comparar la huella.

## Cómo se instala o se quita

El modelo viene en `datos/modelos/` y la imagen de Docker lo incluye. Si esa carpeta no está (o se
indica otra con `FARO_MODELOS`), el portal sigue funcionando: registra las visitas y las cruza con
la posición, el satélite y el censo, y avisa que la foto no se evaluó.
