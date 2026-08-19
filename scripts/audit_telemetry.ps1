# Titan Browser - Real-Time Telemetry & Socket Auditor
# Usage: .\scripts\audit_telemetry.ps1

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "  🛡️  Titan Browser - Real-Time Telemetry & Socket Auditor  " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host ""

$telemetryKeywords = @(
    "aria", "telemetry", "events.data", "watson", "analytics",
    "doubleclick", "sentry", "mixpanel", "segment", "clarity",
    "scorecardresearch", "hotjar", "datadog"
)

function Get-TitanPids {
    $pids = @()
    $titanProc = Get-Process -Name "titan-browser" -ErrorAction SilentlyContinue
    if ($titanProc) {
        $pids += $titanProc.Id
        $children = Get-CimInstance Win32_Process | Where-Object { $_.ParentProcessId -in $titanProc.Id }
        if ($children) {
            $pids += $children.ProcessId
        }
    }
    return $pids
}

$activePids = Get-TitanPids
if ($activePids.Count -eq 0) {
    Write-Host "[!] Titan Browser is not currently running. Please launch it with 'cargo run' first." -ForegroundColor Yellow
    exit 0
}

Write-Host "[+] Found Titan Browser running with PID(s): $($activePids -join ', ')" -ForegroundColor Green
Write-Host "[+] Monitoring live TCP/UDP socket activity (Press Ctrl+C to stop)..." -ForegroundColor Gray
Write-Host ""

$seenConnections = @{}

try {
    while ($true) {
        $activePids = Get-TitanPids
        if ($activePids.Count -eq 0) {
            Write-Host "[*] Titan Browser process exited. Stopping monitor." -ForegroundColor Yellow
            break
        }

        $conns = Get-NetTCPConnection -ErrorAction SilentlyContinue | Where-Object { $_.OwningProcess -in $activePids }

        if ($conns.Count -eq 0) {
            Write-Host -NoNewline "`r[*] 0 Outbound Sockets Active (Idle / Local-only state)    " -ForegroundColor DarkGray
        } else {
            foreach ($conn in $conns) {
                $connKey = "$($conn.RemoteAddress):$($conn.RemotePort)"
                if (-not $seenConnections.ContainsKey($connKey) -and $conn.RemoteAddress -ne "127.0.0.1" -and $conn.RemoteAddress -ne "0.0.0.0" -and $conn.RemoteAddress -ne "::1") {
                    $seenConnections[$connKey] = $true
                    
                    # Reverse DNS lookup
                    $remoteHost = $conn.RemoteAddress
                    try {
                        $dns = [System.Net.Dns]::GetHostEntry($conn.RemoteAddress)
                        if ($dns -and $dns.HostName) {
                            $remoteHost = $dns.HostName
                        }
                    } catch {}

                    # Check for telemetry keywords
                    $isTelemetry = $false
                    foreach ($kw in $telemetryKeywords) {
                        if ($remoteHost -like "*$kw*") {
                            $isTelemetry = $true
                            break
                        }
                    }

                    $timestamp = (Get-Date).ToString("HH:mm:ss")
                    if ($isTelemetry) {
                        Write-Host "`n[$timestamp] ⚠️ TELEMETRY PACKET DETECTED: $remoteHost ($($conn.RemoteAddress):$($conn.RemotePort)) [PID: $($conn.OwningProcess)]" -ForegroundColor Red
                    } else {
                        Write-Host "`n[$timestamp] 🌐 Outbound Connection: $remoteHost ($($conn.RemoteAddress):$($conn.RemotePort)) [PID: $($conn.OwningProcess)] - State: $($conn.State)" -ForegroundColor Green
                    }
                }
            }
        }

        Start-Sleep -Milliseconds 400
    }
} catch {
    Write-Host "`n[+] Auditor stopped." -ForegroundColor Cyan
}
