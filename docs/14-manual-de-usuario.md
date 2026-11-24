# Manual de usuario

FARO reúne en una sola pantalla lo que hoy está repartido en correos, hojas de cálculo y portales:
**pedir y recibir registros de otras instituciones, entenderlos sin adivinar, y aprovecharlos junto a
los datos del propio INE**. Se abre en el navegador (<http://localhost:8082>), funciona en laptop,
tableta y celular, y cada cifra dice de dónde sale.

## Primer ingreso

Entre con `admin` / `admin`. El portal le pedirá cambiar la contraseña en «Mi perfil» y, por
seguridad, no dejará hacer cambios hasta que lo haga. Después cree a las personas en «Usuarios».

| Rol | Puede |
|---|---|
| Administrador | Todo, incluido manejar usuarios y grupos |
| Analista | Recibir envíos, decidir pendientes, manejar precios, muestreos y tableros |
| Consulta | Ver |

Los **grupos** limitan a qué secciones entra cada quien (por ejemplo, «Precios e índices» no ve la
recepción de registros). El menú de cada persona muestra solo lo suyo.

## Lo que resuelve, sección por sección

### Registros administrativos: de un archivo cualquiera a un dato oficial
1. **Recibir envío.** Elija la solicitud y suba el Excel o CSV tal como lo tiene la institución. En
   segundos sabe qué filas sirven, cuáles no y por qué. El archivo original no se guarda.
2. **Revisar pendientes.** Lo que el sistema no quiso adivinar. Usted elige una vez; queda aprendido
   para todas las instituciones, con su nombre y fecha.
3. **Equivalencias.** La memoria institucional: todo lo aprendido, y la carga de tablas de códigos propias.
4. **Resultados y avance.** Qué se pidió, qué llegó, con qué calidad y cómo va cada departamento.

### Territorio: el país hasta el lugar poblado
- **Mapa** en tres niveles con buscador, ficha, vista aislada y reporte imprimible. La ficha de un
  municipio trae población, Censo 2018 por lugar poblado, pirámide de edades y crecimiento construido.
- **Población y crecimiento**, con el cálculo repetible paso a paso y márgenes comprobados contra lo ocurrido.
- **Red vial**: lugares sin conexión y enlaces que probablemente faltan en el mapa.
- **Operadores de campo**: reparto con la menor distancia total.
- **Seguridad alimentaria**: datos de SESAN por departamento y municipio.

### Precios: evidencia para cada precio
- **Precios por rubro** (con su menú): más de 85 mil precios publicados; el mismo producto entre
  tiendas por código de barras; diferencias grandes explicadas o por confirmar; **informe de precio
  de referencia** listo para imprimir, con foto, intervalo de confianza y fuentes.
- **Materiales de construcción** frente a la canasta del IPMC; **precios de referencia** desde
  cotizaciones; **canasta básica** con su histórico; **hoja de balance** con tasas oficiales.
- **Muestreos automáticos**: el portal vuelve a leer las tiendas solo, cuando usted decida.

### Tableros a pedido
Escriba lo que necesita —«mapa y tabla de desnutrición y población en Sololá, los 5 más altos»— y
el portal arma el tablero y le dice qué entendió. Edítelo pieza por pieza, guárdelo, imprímalo.

### Datos y transparencia
**Portal del INE** (conjuntos vigilados), **capas publicadas** para cualquier SIG, **proceso** (qué
hace cada etapa y quién) y **acerca y fuentes** (cada dato con su cita).

## Tres garantías

- **No adivina.** Asigna un código solo con coincidencia exacta y única, o con una decisión humana registrada.
- **No inventa.** Todo dato es real, con fuente, fecha y licencia; lo calculado se puede repetir con un comando.
- **Se revisa a sí mismo.** Verificación estadística, de interfaz y de seguridad, con sus informes en `docs/19` a `docs/21`.
