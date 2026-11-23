# Crea .env con claves aleatorias si no existe y deja en FARO_PUERTO un puerto libre.
# Lo llama iniciar.bat.
$PuertoInicial = 8090
$ruta = Join-Path (Get-Location) ".env"

function Clave { -join ((48..57) + (97..122) | Get-Random -Count 32 | ForEach-Object { [char]$_ }) }

# Un puerto está libre si esta máquina deja abrirlo para escuchar.
function PuertoLibre([int]$puerto) {
    try {
        $escucha = New-Object System.Net.Sockets.TcpListener([System.Net.IPAddress]::Any, $puerto)
        $escucha.Start()
        $escucha.Stop()
        return $true
    } catch {
        return $false
    }
}

function PrimerPuertoLibre([int]$desde) {
    foreach ($p in $desde..($desde + 100)) {
        if (PuertoLibre $p) { return $p }
    }
    throw "No hay un puerto libre entre $desde y $($desde + 100)."
}

if (-not (Test-Path $ruta)) {
    $lineas = @(
        "POSTGRES_PASSWORD=$(Clave)",
        "FARO_TOKEN=$(Clave)",
        "FARO_SAL=$(Clave)",
        "FARO_SYNC_HORAS=24",
        "FARO_PUERTO=$(PrimerPuertoLibre $PuertoInicial)"
    )
    [System.IO.File]::WriteAllLines($ruta, $lineas)
    Write-Output "Se creo .env con claves nuevas."
    exit 0
}

# .env ya existe: las claves no se tocan. Solo se cambia el puerto si lo ocupa otro programa.
$lineas = @(Get-Content $ruta)
$puerto = $PuertoInicial
$tiene = $false
foreach ($l in $lineas) {
    if ($l -match '^FARO_PUERTO=(\d+)\s*$') { $puerto = [int]$Matches[1]; $tiene = $true }
}

# Si quien ocupa el puerto es este mismo FARO (ya levantado desde esta carpeta), se conserva.
$propio = ""
try { $propio = (& docker compose port faro 8080 2>$null | Out-String).Trim() } catch { $propio = "" }
$esPropio = $propio -match ":$puerto$"

if ($esPropio -or (PuertoLibre $puerto)) {
    if (-not $tiene) {
        [System.IO.File]::WriteAllLines($ruta, [string[]]($lineas + "FARO_PUERTO=$puerto"))
    }
    Write-Output "FARO usara el puerto $puerto."
} else {
    $nuevo = PrimerPuertoLibre $PuertoInicial
    $lineas = @($lineas | Where-Object { $_ -notmatch '^FARO_PUERTO=' }) + "FARO_PUERTO=$nuevo"
    [System.IO.File]::WriteAllLines($ruta, [string[]]$lineas)
    Write-Output "El puerto $puerto esta ocupado por otro programa. FARO usara el puerto $nuevo."
}
