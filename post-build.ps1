$target = "target"

$exe = "blackout.exe"
$echo = "echo.exe"

$configs = @("debug", "release")

$deploy = "deploy"

$installer = "install.ps1"
$uninstaller = "uninstall.ps1"

foreach ($config in $configs)
{
    $tc = Join-Path -Path $target -ChildPath $config

    if (Test-Path -Path $tc -PathType Container)
    {
        $dd = Join-Path -Path $deploy -ChildPath $config

        if (-not (Test-Path -Path $dd -PathType Container))
        {
            New-Item -Path $dd -ItemType Directory | Out-Null
        }

        $sf = Join-Path -Path $tc -ChildPath $exe
        $df = Join-Path -Path $dd -ChildPath $exe
        Copy-Item -Path $sf -Destination $df -Force

        $se = Join-Path -Path $tc -ChildPath $echo
        $de = Join-Path -Path $dd -ChildPath $echo
        Copy-Item -Path $se -Destination $de -Force

        $si = Join-Path "src/installer" -ChildPath $installer
        $di = Join-Path -Path $dd -ChildPath $installer
        Copy-Item -Path $si -Destination $di -Force

        $su = Join-Path "src/installer" -ChildPath $uninstaller
        $du = Join-Path $dd -ChildPath $uninstaller
        Copy-Item -Path $su -Destination $du -Force
    }
}
