const { chromium } = require('playwright');
(async () => {
  const [,, base, token, archivo, dir] = process.argv;
  const b = await chromium.launch(); const p = await b.newPage({ viewport: { width: 1380, height: 950 } });
  const errores = []; p.on('pageerror', e => errores.push(String(e))); p.on('console', m => m.type() === 'error' && errores.push(m.text()));
  const enviar = async () => { await p.click('nav button[data-s=recibir]'); await p.selectOption('#sol', 'hechos-transito-v1'); await p.setInputFiles('#archivo', archivo); await p.click('#enviar'); await p.waitForTimeout(1200); return (await p.textContent('#resultado')).replace(/\s+/g, ' ').slice(0, 330); };
  await p.goto(base); await p.waitForSelector('#sol option', {state:'attached'}); await p.fill('#token', token);
  console.log('1er envío:', await enviar());
  await p.click('nav button[data-s=pendientes]'); await p.waitForSelector('#tpend tbody tr'); await p.fill('#usuario', 'analista de prueba');
  const filas = await p.locator('#tpend tbody tr').allTextContents(); filas.forEach(f => console.log('  pend:', f.replace(/\s+/g, ' ')));
  await p.screenshot({ path: dir + '/5-pendientes.png', fullPage: true });
  // código 37 en Escuintla: buscar en la lista del catálogo
  const campo = p.locator('#tpend tr', { hasText: '37' }).locator('input.buscar'); await campo.focus(); await p.waitForTimeout(600);
  console.log('opciones Escuintla:', await p.locator('datalist#cat-municipio-05 option').count());
  await campo.fill('0510 Iztapa'); await campo.dispatchEvent('change'); await p.waitForTimeout(700);
  await p.locator('#tpend tr', { hasText: 'Camotan' }).locator('button[data-c]').first().click(); await p.waitForTimeout(700);
  await p.locator('#tpend tr', { hasText: 'Cuatrimoto' }).locator('button', { hasText: 'otro' }).click(); await p.waitForTimeout(700);
  await p.locator('#tpend tr', { hasText: 'Motobici' }).locator('button', { hasText: 'motocicleta' }).click(); await p.waitForTimeout(700);
  console.log('quedan:', (await p.locator('#tpend tbody tr').allTextContents()).map(f => f.replace(/\s+/g, ' ')));
  console.log('2do envío:', await enviar());
  await p.click('nav button[data-s=memoria]'); await p.waitForTimeout(500); console.log('equivalencias:', (await p.locator('#tmem tbody tr').allTextContents()).map(f => f.replace(/\s+/g, ' ').slice(0, 70)));
  await p.click('nav button[data-s=resultados]'); await p.waitForTimeout(800); await p.screenshot({ path: dir + '/6-resultados.png', fullPage: true });
  console.log('hechos:', (await p.locator('#thechos tbody tr').allTextContents()).map(f => f.replace(/\s+/g, ' ')));
  console.log('avance:', await p.locator('#tavance tbody tr').count());
  await p.click('nav button[data-s=crecimiento]'); await p.waitForSelector('#tpob tbody tr td b'); console.log('pob:', await p.textContent('#pobpais'), '|', (await p.locator('#tpob tbody tr').first().textContent()).replace(/\s+/g, ' '), '| filas', await p.locator('#tpob tbody tr').count());
  await p.screenshot({ path: dir + '/7-crecimiento.png' });
  const r = await p.request.get(base + '/ejemplos/edificaciones_ficticias.csv'); console.log('ejemplo:', r.status(), (await r.text()).slice(0, 25).replace(/\n/g, ' '), '| malo:', (await p.request.get(base + '/ejemplos/..%2F..%2Fetc%2Fpasswd')).status());
  console.log('errores js:', JSON.stringify(errores)); await b.close();
})();
