// Edificaciones por municipio y año para Guatemala (2016–2023).
//
// Dónde corre: Google Earth Engine, editor de código (https://code.earthengine.google.com).
// Earth Engine es un servicio en la nube de Google: su uso se declara en el README.
//
// Datos:
//   - Google Research, Open Buildings 2.5D Temporal v1 (CC BY 4.0 / ODbL).
//     ee.ImageCollection("GOOGLE/Research/open-buildings-temporal/v1")
//   - Límites municipales: FAO GAUL 2015 nivel 2. Sus nombres NO traen código INE:
//     el CSV resultante se pasa por `faro recibir` para homologarlos al código oficial.
//
// Resultado: un CSV en Google Drive con departamento, municipio, anio, edificaciones.
// Estado: escrito el 23-nov-2026; pendiente de ejecutar y validar por el equipo.

var municipios = ee.FeatureCollection('FAO/GAUL/2015/level2')
  .filter(ee.Filter.eq('ADM0_NAME', 'Guatemala'));

var coleccion = ee.ImageCollection('GOOGLE/Research/open-buildings-temporal/v1')
  .filterBounds(municipios);

var anios = ee.List.sequence(2016, 2023);

var porAnio = anios.map(function (anio) {
  anio = ee.Number(anio).int();
  // Cada imagen anual está fechada a mitad de año.
  var imagen = coleccion
    .filterDate(ee.Date.fromYMD(anio, 1, 1), ee.Date.fromYMD(anio, 12, 31))
    .select('building_fractional_count')
    .mosaic();
  // La suma de "conteo fraccional" sobre un área estima su número de edificaciones.
  var sumas = imagen.reduceRegions({
    collection: municipios,
    reducer: ee.Reducer.sum(),
    scale: 4,
    tileScale: 8
  });
  return sumas.map(function (f) {
    return ee.Feature(null, {
      departamento: f.get('ADM1_NAME'),
      municipio: f.get('ADM2_NAME'),
      anio: anio,
      edificaciones: f.get('sum')
    });
  });
});

Export.table.toDrive({
  collection: ee.FeatureCollection(porAnio).flatten(),
  description: 'edificaciones_por_municipio_gt_2016_2023',
  fileFormat: 'CSV',
  selectors: ['departamento', 'municipio', 'anio', 'edificaciones']
});
