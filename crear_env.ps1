# Crea .env con claves aleatorias si no existe. Lo llama iniciar.bat.
if (-not (Test-Path ".env")) {
    function Clave { -join ((48..57) + (97..122) | Get-Random -Count 32 | ForEach-Object { [char]$_ }) }
    $lineas = @(
        "POSTGRES_PASSWORD=$(Clave)",
        "FARO_TOKEN=$(Clave)",
        "FARO_SAL=$(Clave)",
        "FARO_SYNC_HORAS=24",
        "FARO_PUERTO=8080"
    )
    [System.IO.File]::WriteAllLines((Join-Path (Get-Location) ".env"), $lineas)
    Write-Output "Se creo .env con claves nuevas."
}
