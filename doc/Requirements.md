# Requirements

## Functional

| ID | Name                         | Desc                                                                           | Status  | 
|----|------------------------------|--------------------------------------------------------------------------------|---------|
| 1  | Run at startup               | The app must start running in the background when OS boots                     | &check; |
| 2  | Listen for mouse gestures    | Implement mouse gestures recognition (arm & trigger)                           | &check; |
| 3  | Sound confirmations          | The app should play an audio confirmations (since the screen is supposed dead) | &check; |
| 4  | Visual backup status         | Display the status of the backup in progress                                   | &check; |
| 5  | Backup source(s) description | Parse a configuration file in which to specify the backup sources              | &check; |
| 6  | Find suitable drive          | Find a removable drive (USB) that has enough free space to store the backup    | &check; |
| 7  | File copying                 | Implement the file copying logic                                               | &check; |
| 8  | Monitor CPU usage            | Implement CPU usage monitoring                                                 | &check; |
| 9  | Log files                    |                                                                                | &check; |
|    | 9.1                          | Save a local log of CPU usage every 2 minutes                                  | &check; |
|    | 9.2                          | Save a log on dest of the backupped files size and CPU time elapsed            | &check; |

## Non Functional

| ID | Name                     | Desc                                                                 | Status  |
|----|--------------------------|----------------------------------------------------------------------|---------|
| 1  | Least CPU usage possible | Optimize busy waiting and execution efficiency to minimize CPU usage | &check; |
