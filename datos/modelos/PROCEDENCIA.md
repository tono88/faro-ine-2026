# Procedencia del modelo de fotos (`places365_resnet18.onnx`)

Modelo de clasificación de escenas **Places365-ResNet18**, de MIT CSAIL, exportado a un solo archivo
ONNX para correrlo dentro del servidor con `tract` (Rust puro). FARO no lo entrenó ni lo ajustó:
lo usa tal cual para decir si la foto de una visita de campo parece una vivienda, un comercio u
otra cosa. Todo se descargó y se generó el **24-nov-2026**.

## Archivos de esta carpeta

| Archivo | Bytes | SHA-256 | De dónde sale |
|---|---|---|---|
| `places365_resnet18.onnx` | 45,445,537 | `60154904f0ee5d8f1dc25f62bdbbc8c4125fb8e8b3dccc3faf688ac4b3bc05ab` | Exportado con `scripts/modelo/exportar.py` a partir de los pesos de los autores |
| `categories_places365.txt` | 6,833 | `2affba635eb657e7ca95f4e6cc69bd9fac29ef4c32aeb83cafdfcd06ec6a1ea6` | <https://raw.githubusercontent.com/CSAILVision/places365/master/categories_places365.txt> (commit `8a953ed`) |

Forma del modelo: entrada `imagen`, float32 `[1, 3, 224, 224]`; salida `puntajes`, 365 valores (las
probabilidades se obtienen con softmax). Opset 12; operadores Conv, Relu, Add, MaxPool,
GlobalAveragePool, Flatten y Gemm; los pesos van dentro del archivo.

## Licencia y cita

- **Pesos preentrenados: Creative Commons Atribución (CC BY).** El repositorio de los autores dice:
  «The pretrained places-CNN models can be used under the Creative Common License (Attribution
  CC BY). Please give appropriate credit, such as providing a link to our paper or to the Places
  Project Page» (<https://github.com/CSAILVision/places365#acknowledgements-and-license>).
- **Código de Places365: MIT** («Copyright (c) 2017 Bolei Zhou and MIT CSAIL Computer Vision»).
- **Cita pedida por los autores:** Zhou, B.; Lapedriza, A.; Khosla, A.; Oliva, A.; Torralba, A.
  (2017). «Places: A 10 million Image Database for Scene Recognition». *IEEE Transactions on
  Pattern Analysis and Machine Intelligence*. Página del proyecto: <http://places2.csail.mit.edu>.
- El archivo ONNX es una obra derivada de esos pesos: se distribuye con el mismo crédito.

## De dónde se bajaron los pesos (y por qué se confía en la copia)

El sitio oficial (`places2.csail.mit.edu`) no respondió desde la nube donde se trabajó. Se usó una
copia del mismo archivo alojada en GitHub:

- `resnet18_places365.pth.tar`, 45,506,139 bytes, SHA-256
  `2f4759217d470da2b803f8f66cd4488a066406b555a5fb95ee9a4663f9f05588`, tomado de
  <https://raw.githubusercontent.com/Pahul0516/Scene_CNN_App/e4871ed198bdf604ee7935e1bc0333153db12d2c/resources/resnet18_places365.pth.tar>.
  Ese repositorio es de un tercero y solo sirvió de espejo.

Comprobaciones de que es el archivo de los autores:

1. El mismo archivo (mismo tamaño y mismo identificador de contenido de git) está en cuatro
   repositorios públicos independientes.
2. Sus 102 tensores están, byte a byte, dentro de la copia que redistribuye IBM en su Model Asset
   Exchange (`MAX-Scene-Classifier`), cuyo SHA-512 coincide con el que IBM publica.
3. Sobre las 36,500 imágenes de validación oficiales de Places365 da **53.6 % de acierto en la
   primera opción y 83.7 % entre las cinco primeras**. El punto de control declara 54.5 %: queda
   una diferencia de 0.9 puntos sin explicar, que no viene de la exportación (PyTorch da lo mismo).
4. El archivo se cargó sin ejecutar código (`weights_only=True`).

**Recomendación:** antes de un uso real, bajar los pesos del sitio oficial desde la red del INE y
comparar su SHA-256 con el de arriba.

## Cómo se exportó

```
python3 -m venv venv
venv/bin/pip install --index-url https://download.pytorch.org/whl/cpu torch torchvision
venv/bin/pip install onnx onnxruntime pillow
venv/bin/python -I scripts/modelo/exportar.py --pesos resnet18_places365.pth.tar --salida datos/modelos/places365_resnet18.onnx
```

Versiones usadas: Python 3.13, torch 2.15.0, torchvision 0.29.2, onnx 1.23.2, onnxruntime 1.32.0.
Con esas versiones la exportación da el mismo SHA-256. Diferencia máxima entre PyTorch y el ONNX:
6.7 × 10⁻⁶; entre el ONNX en `tract` 0.23 y en ONNX Runtime: 8.1 × 10⁻⁶.

## Cómo se prepara la foto

Lado corto a 256 píxeles (filtro bilineal), recorte central de 224 × 224, colores entre 0 y 1 y
normalizados con media `[0.485, 0.456, 0.406]` y desviación `[0.229, 0.224, 0.225]` (rojo, verde,
azul). Es la transformación de validación de los autores. Está en `preparar()`, en
`crates/faro-fuentes/src/fachada.rs`.

## Cómo se usa y qué tan bien funciona

Ver [`docs/24-clasificador-de-fotos.md`](../../docs/24-clasificador-de-fotos.md).
