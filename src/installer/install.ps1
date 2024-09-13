$SCRIPT_DIR = Split-Path -Parent $MyInvocation.MyCommand.Definition

$EXECUTABLE_PATH = Join-Path $SCRIPT_DIR "blackout.exe"

Write-Output "Creating startup task for $EXECUTABLE_PATH ..."

$TASK_NAME = "BlackoutStart"

$action = New-ScheduledTaskAction -Execute $EXECUTABLE_PATH -WorkingDirectory $SCRIPT_DIR

$trigger = New-ScheduledTaskTrigger -AtLogOn

$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable

$principal = New-ScheduledTaskPrincipal -UserId "NT AUTHORITY\SYSTEM" -LogonType ServiceAccount

Register-ScheduledTask -TaskName $TASK_NAME -Action $action -Trigger $trigger -Settings $settings -Principal $principal