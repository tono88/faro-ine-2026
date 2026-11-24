# Manual de usuario

FARO se abre en el navegador (por omisión <http://localhost:8082>). Se puede ver todo sin entrar;
para **cambiar** algo (recibir un envío, decidir un pendiente, ejecutar un muestreo) hay que
iniciar sesión. Usuario inicial: `admin` / `admin`; el portal pide cambiar la contraseña.

| Rol | Puede |
|---|---|
| Administrador | Todo, incluido crear y desactivar usuarios |
| Analista | Recibir envíos, decidir pendientes, cargar tablas, manejar muestreos y precios |
| Consulta | Solo ver |

## Registros administrativos

1. **Recibir envío.** Elija la solicitud (qué se le pidió a la institución), descargue la plantilla
   si la institución aún no tiene el formato, y suba el archivo (Excel o CSV). El portal lo valida
   contra la solicitud, homologa los lugares y muestra el resultado. El archivo original no se guarda.
2. **Revisar pendientes.** Lo que el sistema no quiso adivinar. Elija a qué corresponde cada texto
   buscando en el catálogo; la decisión queda grabada con su nombre y no se vuelve a preguntar.
3. **Equivalencias.** Todo lo aprendido; aquí se carga también una tabla de códigos propia de una
   institución.
4. **Resultados y avance.** Qué se pidió, qué llegó y con qué calidad; cobertura por departamento
   (22 delegaciones) y conteos con tasa por 100 mil habitantes.

## Territorio

- **Mapa.** Tres niveles (departamentos, municipios, lugares poblados). Elija qué colorear, toque
  un territorio para su ficha, use el buscador para ir a un lugar por nombre o código, «Ver solo
  este…» para aislarlo y ⌂ para volver al país. La ficha del municipio trae población, censo 2018,
  pirámide de edades, superficie construida y un reporte para imprimir.
- **Población y crecimiento.** Proyecciones del INE y superficie construida vista por satélite; el
  cálculo de población se puede repetir desde la pantalla, paso a paso.
- **Red vial.** Lugares poblados sin conexión y enlaces que probablemente faltan; «Ver en el mapa»
  lleva al enlace.
- **Operadores de campo.** Suba operadores y puestos; el portal los reparte con la menor distancia total.
- **Seguridad alimentaria.** Inseguridad alimentaria aguda por departamento y desnutrición crónica
  en escolares por municipio (SESAN).

## Precios

- **Materiales de construcción.** Productos publicados por proveedores frente a la canasta del IPMC.
- **Precios de referencia.** Suba cotizaciones y obtenga media, mediana, moda y el precio
  recomendado; busque antecedentes en los precios ya emitidos por el INE.
- **Canasta básica.** Precio promedio del INE por producto; toque la gráfica para ampliar el histórico.
- **Precios por rubro** (y el menú de rubros debajo). Busque un producto; vea el mismo producto
  entre tiendas por código de barras; revise las **diferencias de 50 % o más** (el portal dice si
  es una oferta, una tienda atípica o si necesita su confirmación: «es correcto» / «es un error»);
  haga el **informe de precio de referencia** de una búsqueda.
- **Hoja de balance: derivados.** Elija el producto primario y la cantidad procesada.
- **Muestreos automáticos.** Encienda cada tarea, cambie su frecuencia, ejecútela ahora o use
  «Ejecutar todas ahora».

## Datos

- **Portal del INE.** Conjuntos del portal de datos abiertos vigilados por FARO.
- **Capas publicadas.** Direcciones estables (GeoJSON y CSV) para un SIG u otro portal.
- **Acerca y fuentes.** Qué es, cómo decide y las citas de todo lo usado.

Junto a cada resultado hay un enlace para descargar el procedimiento que lo reproduce fuera del portal.
